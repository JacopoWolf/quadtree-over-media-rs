use crate::{DEFAULT_COLOR, DEFAULT_TRESHOLD};
use image::{DynamicImage, GenericImage, GenericImageView, Rgba, RgbaImage};
use std::collections::VecDeque;

use crate::utils::Vec2;

const MIN_SIZE: Vec2 = Vec2 { x: 3, y: 3 };

//TODO allow 3x3 and 4x4 quads

/// Draws quads based on the specified image and with the given args
pub fn draw_quads_on_image(img: &DynamicImage, args: &super::QuadArgs) -> DynamicImage {
    let mut imgcopy = if args.quads_only {
        DynamicImage::ImageRgba8(RgbaImage::new(img.width(), img.height()))
    } else {
        img.clone()
    };
    let mut curr_depth: u32 = 1;
    let max_depth = ((img.width() * img.height()) as f64).log2() as u32 / 2;
    println!("Max iterations: {max_depth}");

    let mut queue_in: VecDeque<Vec2> = VecDeque::from([Vec2::new()]);
    let mut queue_out: VecDeque<Vec2> = VecDeque::with_capacity(4);

    while curr_depth < max_depth && queue_in.len() > 0 {
        // halves size at each iteration
        let curr_size = Vec2 {
            x: img.width() >> curr_depth,
            y: img.height() >> curr_depth,
        };

        if curr_size.smaller(&MIN_SIZE) {
            println!("reached minimum possible quad size!");
            break;
        }
        println!("Iteration: {}, size {}", curr_depth, curr_size);

        while !queue_in.is_empty() {
            //TODO this could be HEAVILY parallelized
            let curr_node = queue_in.pop_front().unwrap();

            let pos_tmp = [
                Vec2 {
                    x: curr_node.x,
                    y: curr_node.y,
                },
                Vec2 {
                    x: curr_node.x + curr_size.x,
                    y: curr_node.y,
                },
                Vec2 {
                    x: curr_node.x,
                    y: curr_node.y + curr_size.y,
                },
                Vec2 {
                    x: curr_node.x + curr_size.x,
                    y: curr_node.y + curr_size.y,
                },
            ];

            if curr_depth > args.min_depth {
                let averages = [
                    average_colors(&img, &pos_tmp[0], &curr_size),
                    average_colors(&img, &pos_tmp[1], &curr_size),
                    average_colors(&img, &pos_tmp[2], &curr_size),
                    average_colors(&img, &pos_tmp[3], &curr_size),
                ];
                if are_le_treshold(&averages, &args.treshold.unwrap_or(DEFAULT_TRESHOLD)) {
                    continue;
                }
            }

            for i in 0..4 {
                draw_square(
                    &args.color.unwrap_or(DEFAULT_COLOR),
                    &mut imgcopy,
                    &pos_tmp[i],
                    &curr_size,
                );
                queue_out.push_back(pos_tmp[i].clone());
            }
        }

        curr_depth += 1;
        queue_in = queue_out;
        queue_out = VecDeque::with_capacity(4);
    }

    imgcopy
}

/// if all the differences between each max and min RGBA  LESS than the treshold
fn are_le_treshold(sub_averages: &[[u8; 4]; 4], treshold: &Rgba<u8>) -> bool {
    (0..=3) // index R, G, B, A
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

fn draw_square(color: &Rgba<u8>, img: &mut DynamicImage, pos: &Vec2, size: &Vec2) -> () {
    for x in 0..size.x {
        img.put_pixel(pos.x + x, pos.y, *color);
    }
    for y in 0..size.y {
        img.put_pixel(pos.x, pos.y + y, *color);
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
