use std::collections::HashMap;

use crate::utils::*;
use image::*;
use rayon::prelude::*;

pub(super) const DEFAULT_MIN_DEPTH: u8 = 4;
pub(super) const DEFAULT_COLOR: image::Rgba<u8> = image::Rgba([255, 20, 147, 255]); //DeepPink
pub(super) const DEFAULT_TRESHOLD: image::Rgba<u8> = image::Rgba([8, 8, 8, 8]);
pub(super) const DEFAULT_MIN_SIZE: Vec2 = Vec2 { x: 3, y: 3 };

pub fn calc_quads(
    img: &DynamicImage,
    min_quad_size: &Vec2,
    min_depth: u8,
    treshold: &Rgba<u8>,
    do_calc_color: bool,
) -> (HashMap<Vec2, QuadInfo>, HashMap<u8, Vec2>) {
    let max_depth = ((img.width() * img.height()) as f64).log2() as u8 / 2;
    println!("Max iterations: {max_depth}");

    let mut quadinf_map: HashMap<Vec2, QuadInfo> = HashMap::new();
    let mut depth_sizeinf_map: HashMap<u8, Vec2> = HashMap::new();

    let mut curr_size = Vec2 {
        x: img.width(),
        y: img.height(),
    };
    depth_sizeinf_map.insert(0, curr_size);
    quadinf_map.insert(
        Vec2::ZERO,
        QuadInfo {
            depth: 0,
            color: None,
        },
    );

    let mut curr_depth: u8 = 1;
    let mut quadpos_in: Vec<Vec2> = vec![Vec2::ZERO];
    let mut quadinf_out: Vec<(Vec2, QuadInfo)>;

    while curr_depth < max_depth && !quadpos_in.is_empty() {
        // halves size at each iteration
        let h = curr_size.half();
        curr_size = h.0;
        if &curr_size < min_quad_size {
            println!("reached minimum possible quad size!");
            break;
        }
        depth_sizeinf_map.insert(curr_depth, curr_size);

        println!("Iteration: {}, size {}, mod {}", curr_depth, curr_size, h.1);

        quadinf_out = Vec::from_par_iter(
            quadpos_in
                .par_iter()
                .map(|node| -> Option<[(Vec2, QuadInfo); 4]> {
                    let mut subs = generate_subnodes(node, &curr_size, &h.1, curr_depth);
                    if curr_depth > min_depth || do_calc_color {
                        let averages = [
                            average_colors(&img, &subs[0].0, &curr_size),
                            average_colors(&img, &subs[1].0, &curr_size),
                            average_colors(&img, &subs[2].0, &curr_size),
                            average_colors(&img, &subs[3].0, &curr_size),
                        ];
                        if are_le_treshold(&averages, treshold) {
                            return None;
                        }
                        // assign colors
                        if do_calc_color {
                            subs[0].1.color = Some(image::Rgba(averages[0]));
                            subs[1].1.color = Some(image::Rgba(averages[1]));
                            subs[2].1.color = Some(image::Rgba(averages[2]));
                            subs[3].1.color = Some(image::Rgba(averages[3]));
                        }
                    }
                    Some(subs)
                })
                .flatten() // flats Option, removes None
                .flatten(), // SelectMany
        );
        quadpos_in = Vec::with_capacity(quadinf_out.len());
        for (k, v) in quadinf_out {
            quadinf_map.insert(k, v);
            quadpos_in.push(k);
        }
        curr_depth += 1;
    }
    (quadinf_map, depth_sizeinf_map)
}

/// create a copy of the original and draw quads outlines on it
pub fn draw_quads_on(
    original: &DynamicImage,
    quadinf_map: &HashMap<Vec2, QuadInfo>,
    depth_sizeinf_map: &HashMap<u8, Vec2>,
    color: &Option<Rgba<u8>>,
) -> DynamicImage {
    let mut copy = original.clone();

    for (pos, info) in quadinf_map.iter() {
        draw_square(
            &mut copy,
            pos,
            depth_sizeinf_map.get(&info.depth).unwrap(),
            &color.unwrap_or(info.color.unwrap_or(DEFAULT_COLOR)),
            &None,
        );
    }
    copy
}

/// Draws quads based on the specified image and with the given args only if the color satisfies the filter
pub fn draw_quads(
    quadinf_map: &HashMap<Vec2, QuadInfo>,
    depth_sizeinf_map: &HashMap<u8, Vec2>,
    border_color: &Option<Rgba<u8>>,
    background_color: &Option<Rgba<u8>>,
    quad_img: &Option<DynamicImage>,
    draw_range: &Option<[Rgba<u8>; 2]>,
) -> DynamicImage {
    let img_size = depth_sizeinf_map.get(&0).unwrap();
    let mut img_out = DynamicImage::ImageRgba8(match background_color {
        Some(bgrc) => RgbaImage::from_pixel(img_size.x, img_size.y, *bgrc),
        None => RgbaImage::new(img_size.x, img_size.y),
    });

    let mut fillimage_cache: HashMap<Vec2, DynamicImage> = HashMap::new();

    for (pos, info) in quadinf_map.iter() {
        if draw_range.is_some()
            && !color_between(
                &info.color,
                &draw_range.unwrap()[0],
                &draw_range.unwrap()[1],
            )
        {
            continue;
        }
        let size = depth_sizeinf_map.get(&info.depth).unwrap();
        /* increase the size by 1 ther's not a quad there next to this one;
        this check avoids empty line artifacts caused by the modulo
        while halfing odd numbers in the quad size */
        let actual_size: Option<Vec2> = Some(Vec2 {
            // find right
            x: if (pos.x + size.x) < img_size.x {
                match quadinf_map.get(&Vec2 {
                    x: pos.x + size.x,
                    y: pos.y,
                }) {
                    Some(_) => size.x,
                    None => size.x + 1,
                }
            } else {
                size.x
            },
            // find bottom
            y: if (pos.y + size.y) < img_size.y {
                match quadinf_map.get(&Vec2 {
                    x: pos.x,
                    y: pos.y + size.y,
                }) {
                    Some(_) => size.y,
                    None => size.y + 1,
                }
            } else {
                size.y
            },
        });

        //TODO add option to recolor based on info.color
        match quad_img {
            Some(wdy) => draw_image(
                &mut img_out,
                wdy,
                pos,
                actual_size.as_ref().unwrap_or(size),
                &border_color.or(None),
                &mut fillimage_cache,
            ),
            None => {
                draw_square(
                    &mut img_out,
                    pos,
                    actual_size.as_ref().unwrap_or(size),
                    &border_color.unwrap_or(info.color.unwrap_or(DEFAULT_COLOR)),
                    &info.color,
                );
            }
        }
    }
    img_out
}

// create subnodes of the specified size for a given pos and with the given modulo in between
fn generate_subnodes(pos: &Vec2, size: &Vec2, modulo: &Vec2, depth: u8) -> [(Vec2, QuadInfo); 4] {
    [
        (Vec2 { x: pos.x, y: pos.y }, QuadInfo::new(depth)),
        (
            Vec2 {
                x: pos.x + size.x + modulo.x,
                y: pos.y,
            },
            QuadInfo::new(depth),
        ),
        (
            Vec2 {
                x: pos.x,
                y: pos.y + size.y + modulo.y,
            },
            QuadInfo::new(depth),
        ),
        (
            Vec2 {
                x: pos.x + size.x + modulo.x,
                y: pos.y + size.y + modulo.y,
            },
            QuadInfo::new(depth),
        ),
    ]
}

/// if all the differences between each max and min RGBA are LESS than the treshold
fn are_le_treshold(sub_averages: &[[u8; 4]; 4], treshold: &Rgba<u8>) -> bool {
    (0..4) // index R, G, B, A
        .all(|i| {
            let max = sub_averages.iter().map(|&a| a[i]).max().unwrap_or(255);
            let min = sub_averages.iter().map(|&a| a[i]).min().unwrap_or(0);
            return (max - min) <= treshold[i];
        })
}

fn color_between(color: &Option<Rgba<u8>>, from: &Rgba<u8>, to: &Rgba<u8>) -> bool {
    match color {
        Some(color) => (0..4).all(|i| color[i] >= from[i] && color[i] <= to[i]),
        None => false,
    }
    
}

/// calculates the average of each RGBA component individually
fn average_colors(img: &DynamicImage, pos: &Vec2, size: &Vec2) -> [u8; 4] {
    let section = img.view(pos.x, pos.y, size.x, size.y);
    let mut c: u64 = 0;
    let mut tot: [u64; 4] = [0, 0, 0, 0];
    for p in section.pixels() {
        tot[0] += p.2 .0[0] as u64; //R
        tot[1] += p.2 .0[1] as u64; //G
        tot[2] += p.2 .0[2] as u64; //B
        tot[3] += p.2 .0[3] as u64; //A
        c += 1;
    }
    [
        (tot[0] / c) as u8,
        (tot[1] / c) as u8,
        (tot[2] / c) as u8,
        (tot[3] / c) as u8,
    ]
}

// draw the square on the image
fn draw_square(
    img: &mut DynamicImage,
    pos: &Vec2,
    size: &Vec2,
    border_color: &Rgba<u8>,
    fill_color: &Option<Rgba<u8>>,
) -> () {
    unsafe {
        if fill_color.is_some() {
            for y in 1..size.y {
                for x in 1..size.x {
                    img.unsafe_put_pixel(pos.x + x, pos.y + y, fill_color.unwrap())
                }
            }
        }
    }
    // outlines
    for x in 0..size.x {
        img.put_pixel(pos.x + x, pos.y, *border_color);
    }
    for y in 0..size.y {
        img.put_pixel(pos.x, pos.y + y, *border_color);
    }
}

fn draw_image(
    img: &mut DynamicImage,
    img_todraw: &DynamicImage,
    pos: &Vec2,
    size: &Vec2,
    border_color: &Option<Rgba<u8>>,
    cache: &mut HashMap<Vec2, DynamicImage>,
) -> () {
    let draw = match cache.get(size) {
        Some(di) => di,
        None => {
            cache.insert(
                *size,
                img_todraw.resize_exact(size.x, size.y, image::imageops::FilterType::Gaussian),
            );
            cache.get(size).unwrap()
        }
    };
    img.copy_from(draw, pos.x, pos.y)
        .expect("Error while writing sub-image");
    match border_color {
        Some(c) => draw_square(img, pos, size, c, &None),
        None => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_AVERAGES_SIMPLE: [[u8; 4]; 4] = [
        [064, 064, 064, 064],
        [100, 100, 100, 100],
        [100, 100, 100, 100],
        [128, 128, 128, 128],
    ];
    const TEST_AVERAGES_APHAONLY: [[u8; 4]; 4] = [
        [0, 0, 0, 064],
        [0, 0, 0, 100],
        [0, 0, 0, 100],
        [0, 0, 0, 128],
    ];
    #[test]
    fn averages_are_equal_treshold() {
        assert_eq!(
            are_le_treshold(&TEST_AVERAGES_SIMPLE, &Rgba([96, 96, 96, 96])),
            true
        );
    }
    #[test]
    fn averages_are_under_treshold() {
        assert_eq!(
            are_le_treshold(&TEST_AVERAGES_SIMPLE, &Rgba([10, 10, 10, 10])),
            false
        )
    }
    #[test]
    fn alpha_is_under_treshold() {
        assert_eq!(
            are_le_treshold(&TEST_AVERAGES_APHAONLY, &Rgba([255, 255, 255, 96])),
            true
        )
    }
    #[test]
    fn alpha_is_not_under_treshold() {
        assert_eq!(
            are_le_treshold(&TEST_AVERAGES_APHAONLY, &Rgba([255, 255, 255, 97])),
            true
        )
    }
}
