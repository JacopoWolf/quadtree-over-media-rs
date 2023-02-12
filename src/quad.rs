use std::collections::HashMap;

use crate::utils::*;
use image::*;
use log::trace;
use rayon::prelude::*;

pub(super) const DEFAULT_MIN_DEPTH: u8 = 4;
pub(super) const DEFAULT_COLOR: Rgba<u8> = Rgba([255, 20, 147, 255]); //DeepPink
pub(super) const DEFAULT_TRESHOLD: Rgba<u8> = Rgba([8, 8, 8, 8]);
pub(super) const DEFAULT_MIN_SIZE: Vec2 = Vec2 { x: 4, y: 4 };

//TODO add more tests
pub fn calc_quads(
    img: &DynamicImage,
    min_quad_size: &Vec2,
    min_depth: u8,
    treshold: &Rgba<u8>,
    do_calc_color: bool,
) -> QuadStructure {
    trace!(
        "will {} keeping color averages",
        match do_calc_color {
            true => "be",
            false => "not be",
        }
    );

    let max_depth = ((img.width() * img.height()) as f64).log2() as u8 / 2;
    trace!("Max iterations: {max_depth}");

    let mut quadinf_map: HashMap<Vec2, QuadInfo> = HashMap::new();
    let mut depthsizes: Vec<Vec2> = vec![Vec2::ZERO; max_depth as usize];

    let mut curr_size = Vec2::from(img.dimensions());
    depthsizes.insert(0, curr_size);

    let mut curr_depth: u8 = 1;
    let mut quadpos_in: Vec<Vec2> = vec![Vec2::ZERO];
    let mut quadinf_out: Vec<(Vec2, QuadInfo)>;

    while curr_depth < max_depth && !quadpos_in.is_empty() {
        // halves size at each iteration
        let h = curr_size.half();
        curr_size = h.0;
        if &curr_size < min_quad_size {
            trace!("reached minimum possible quad size!");
            break;
        }
        depthsizes.insert(curr_depth as usize, curr_size);

        trace!("Iteration: {}, size {}, mod {}", curr_depth, curr_size, h.1);

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
        // doing it this way avoids concurrency issues. it's still fast af.
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
    QuadStructure {
        quads: quadinf_map,
        sizes: depthsizes,
    }
}

//TODO add tests
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

//TODO add tests
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
    pub use test_case::test_case;

    const BLACK: Rgba<u8> = Rgba::<u8>([0, 0, 0, 0]);

    mod quads {
        use super::*;

        #[test]
        fn gens_only_one_quad() {
            let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(64, 64, BLACK));
            let quadimg = calc_quads(&img, &DEFAULT_MIN_SIZE, 0, &Rgba::<u8>([0, 0, 0, 0]), true);
            assert_eq!(
                quadimg.quads,
                HashMap::from([(
                    Vec2::ZERO,
                    QuadInfo {
                        depth: 0,
                        color: Some(BLACK)
                    }
                )])
            );
            assert_eq!(
                quadimg.sizes,
                vec![
                    Vec2 { x: 64, y: 64 },
                    Vec2 { x: 32, y: 32 },
                    Vec2::ZERO,
                    Vec2::ZERO,
                    Vec2::ZERO,
                    Vec2::ZERO,
                    Vec2::ZERO,
                    Vec2::ZERO
                ]
            )
        }
    }

    mod color_calc {
        use super::{test_case, *};

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

        #[test_case(TEST_AVERAGES_SIMPLE, Rgba([96, 96, 96, 96]) => true; "avg-le")]
        #[test_case(TEST_AVERAGES_SIMPLE, Rgba([10, 10, 10, 10]) => false; "avg-gt")]
        #[test_case(TEST_AVERAGES_APHAONLY, Rgba([255, 255, 255, 65]) => true; "alpha-lt")]
        #[test_case(TEST_AVERAGES_APHAONLY, Rgba([255, 255, 255, 64]) => true; "alpha-eq")]
        #[test_case(TEST_AVERAGES_APHAONLY, Rgba([255, 255, 255, 63]) => false; "alpha-gt")]
        fn treshold(matrix: [[u8; 4]; 4], treshold: Rgba<u8>) -> bool {
            are_le_treshold(&matrix, &treshold)
        }
    }
}
