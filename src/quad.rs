use std::collections::HashMap;

use crate::utils::*;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use rayon::prelude::*;

pub(super) const DEFAULT_MIN_DEPTH: u8 = 4;
pub(super) const DEFAULT_COLOR: image::Rgba<u8> = image::Rgba([255, 20, 147, 255]); //DeepPink
pub(super) const DEFAULT_TRESHOLD: image::Rgba<u8> = image::Rgba([8, 8, 8, 8]);
pub(super) const DEFAULT_MIN_SIZE: Vec2 = Vec2 { x: 3, y: 3 };

/// Draws quads based on the specified image and with the given args
pub fn draw_quads_on_image(img: &DynamicImage, args: &super::QuadArgs) -> DynamicImage {
    let mut imgcopy = if args.no_drawover || args.fill {
        DynamicImage::ImageRgba8(RgbaImage::new(img.width(), img.height()))
    } else {
        img.clone()
    };

    let max_depth = ((img.width() * img.height()) as f64).log2() as u8 / 2;
    println!("Max iterations: {max_depth}");

    let mut curr_depth: u8 = 1;
    let mut quads_map: HashMap<Vec2, QuadInfo> = HashMap::new();
    let mut depths_size_map: HashMap<u8, Vec2> = HashMap::new();
    {
        let mut queue_in: Vec<Vec2> = vec![Vec2 { x: 0, y: 0 }];
        let mut queue_out: Vec<(Vec2, QuadInfo)>;

        while curr_depth < max_depth && !queue_in.is_empty() {
            // halves size at each iteration
            let curr_size = Vec2 {
                x: img.width() >> curr_depth,
                y: img.height() >> curr_depth,
            };
            if curr_size < args.min_quad_size {
                println!("reached minimum possible quad size!");
                break;
            }
            depths_size_map.insert(curr_depth, curr_size);
            println!("Iteration: {}, size {}", curr_depth, curr_size);

            queue_out = Vec::from_par_iter(
                queue_in
                    .par_iter()
                    .map(|node| -> Option<[(Vec2, QuadInfo); 4]> {
                        let mut subs = generate_subnodes(node, &curr_size, curr_depth);
                        if curr_depth > args.min_depth || args.fill {
                            let averages = [
                                average_colors(&img, &subs[0].0, &curr_size),
                                average_colors(&img, &subs[1].0, &curr_size),
                                average_colors(&img, &subs[2].0, &curr_size),
                                average_colors(&img, &subs[3].0, &curr_size),
                            ];
                            if are_le_treshold(
                                &averages,
                                &args.treshold.unwrap_or(DEFAULT_TRESHOLD),
                            ) {
                                return None;
                            }
                            // assign colors
                            if args.fill {
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
            queue_in = Vec::with_capacity(queue_out.len());
            for (k, v) in queue_out {
                quads_map.insert(k, v);
                queue_in.push(k);
            }
            curr_depth += 1;
        }
    }

    // draws each quad
    for (pos, info) in quads_map.iter() {
        //TODO add filter to not draw if the color is too bright or too dark
        //TODO check for is_odd and add 1 pixel to the position of the next quad of bigger size
        draw_square(
            &mut imgcopy,
            &args.color.unwrap_or(info.color.unwrap_or(DEFAULT_COLOR)),
            pos,
            &depths_size_map.get(&info.depth).unwrap(),
            &info.color,
        )
    }
    imgcopy
}

fn generate_subnodes(pos: &Vec2, size: &Vec2, depth: u8) -> [(Vec2, QuadInfo); 4] {
    [
        (Vec2 { x: pos.x, y: pos.y }, QuadInfo::new(depth)),
        (
            Vec2 {
                x: pos.x + size.x,
                y: pos.y,
            },
            QuadInfo::new(depth),
        ),
        (
            Vec2 {
                x: pos.x,
                y: pos.y + size.y,
            },
            QuadInfo::new(depth),
        ),
        (
            Vec2 {
                x: pos.x + size.x,
                y: pos.y + size.y,
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

fn draw_square(
    img: &mut DynamicImage,
    border_color: &Rgba<u8>,
    pos: &Vec2,
    size: &Vec2,
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
        if size.x % 2 == 1 {
            img.put_pixel(pos.x + x, pos.y + size.y, *border_color);
        }
    }
    for y in 0..size.y {
        img.put_pixel(pos.x, pos.y + y, *border_color);
        if size.y % 2 == 1 {
            img.put_pixel(pos.x + size.x, pos.y + y, *border_color);
        }
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
