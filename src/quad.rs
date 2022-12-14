use std::collections::HashMap;

use crate::utils::*;
use image::*;
use rayon::prelude::*;

pub(super) const DEFAULT_MIN_DEPTH: u8 = 4;
pub(super) const DEFAULT_COLOR: Rgba<u8> = Rgba([255, 20, 147, 255]); //DeepPink
pub(super) const DEFAULT_TRESHOLD: Rgba<u8> = Rgba([8, 8, 8, 8]);
pub(super) const DEFAULT_MIN_SIZE: Vec2 = Vec2 { x: 4, y: 4 };

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
    let mut depthsize_map: HashMap<u8, Vec2> = HashMap::new();

    let mut curr_size = Vec2::from(img.dimensions());
    depthsize_map.insert(0, curr_size);

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
        depthsize_map.insert(curr_depth, curr_size);

        println!("Iteration: {}, size {}, mod {}", curr_depth, curr_size, h.1);

        quadinf_out = Vec::from_par_iter(
            quadpos_in
                .par_iter()
                .map(|node| -> Option<[(Vec2, QuadInfo); 4]> {
                    let mut subs = generate_subnodes(node, &curr_size, &h.1, curr_depth);
                    if curr_depth > min_depth || do_calc_color {
                        let averages = [
                            average_colors(img, &subs[0].0, &curr_size),
                            average_colors(img, &subs[1].0, &curr_size),
                            average_colors(img, &subs[2].0, &curr_size),
                            average_colors(img, &subs[3].0, &curr_size),
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

    if do_calc_color && !quadinf_map.contains_key(&Vec2::ZERO) {
        quadinf_map.insert(
            Vec2::ZERO,
            QuadInfo {
                depth: 0,
                color: Some(Rgba(average_colors(
                    img,
                    &Vec2::ZERO,
                    &Vec2::from(img.dimensions()),
                ))),
            },
        );
    }
    (quadinf_map, depthsize_map)
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
            (max - min) <= treshold[i]
        })
}

/// calculates the average of each RGBA component individually
fn average_colors(img: &DynamicImage, pos: &Vec2, size: &Vec2) -> [u8; 4] {
    let section = img.view(pos.x, pos.y, size.x, size.y);
    let mut c: u64 = 0;
    let mut tot: [u64; 4] = [0, 0, 0, 0];
    for (_, _, p) in section.pixels() {
        tot[0] += p[0] as u64; //R
        tot[1] += p[1] as u64; //G
        tot[2] += p[2] as u64; //B
        tot[3] += p[3] as u64; //A
        c += 1;
    }
    [
        (tot[0] / c) as u8,
        (tot[1] / c) as u8,
        (tot[2] / c) as u8,
        (tot[3] / c) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    const BLACK: Rgba<u8> = Rgba::<u8>([0, 0, 0, 0]);
    mod quads {
        use super::*;
        #[test]
        fn gens_only_one_quad() {
            let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, BLACK));
            let (info_map, size_map) =
                calc_quads(&img, &DEFAULT_MIN_SIZE, 0, &Rgba::<u8>([0, 0, 0, 0]), true);
            assert_eq!(
                info_map,
                HashMap::from([(
                    Vec2::ZERO,
                    QuadInfo {
                        depth: 0,
                        color: Some(BLACK)
                    }
                )])
            );
            assert_eq!(
                size_map,
                HashMap::from([(0, Vec2 { x: 64, y: 64 }), (1, Vec2 { x: 32, y: 32 })])
            )
        }
    }
    mod color_calc {
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
}
