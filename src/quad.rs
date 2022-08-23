use image::{DynamicImage, GenericImageView, GenericImage, Rgba, RgbaImage};
use std::collections::VecDeque;
use crate::DEFAULT_COLOR;

use super::utils::Vec2;

const MIN_SIZE: Vec2 = Vec2 { x: 4, y: 4 };
const RGB_TRESHOLD: [u8; 3] = [8, 8, 8];

pub fn draw_quads_on_image(img: &DynamicImage, args: &super::Args) -> DynamicImage {
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
            println!("reached min size!");
            break;
        }
        println!("Iteration: {}, size {}", curr_depth, curr_size);

        while queue_in.len() > 0 {
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
                if is_under_trashold(&averages) {
                    continue;
                }
            }

            for i in 0..4 {
                draw_square(&args.color.unwrap_or(DEFAULT_COLOR), &mut imgcopy, &pos_tmp[i], &curr_size);
                queue_out.push_back(pos_tmp[i].clone());
            }
        }

        curr_depth += 1;
        queue_in = queue_out;
        queue_out = VecDeque::with_capacity(4);
    }

    imgcopy
}


fn is_under_trashold(pixel: &[[u8; 3]; 4]) -> bool {
    //TODO check this and implement a better trashold checking algorithm
    for i in 0..3 {
        let max = pixel.iter().max_by_key(|x| x[i]).unwrap();
        let min = pixel.iter().min_by_key(|x| x[i]).unwrap();
        if (max[i] - min[i]) < RGB_TRESHOLD[i] {
            return true;
        }
    }
    false
}

fn average_colors(img: &DynamicImage, pos: &Vec2, size: &Vec2) -> [u8; 3] {
    let section = img.view(pos.x, pos.y, size.x, size.y);
    let mut c = 0u64;
    let mut tot = [0u64, 0u64, 0u64];
    for p in section.pixels() {
        tot[0] += p.2 .0[0] as u64; //R
        tot[1] += p.2 .0[1] as u64; //G
        tot[2] += p.2 .0[2] as u64; //B
        c += 1;
    }

    [(tot[0] / c) as u8, (tot[1] / c) as u8, (tot[2] / c) as u8]
}

fn draw_square(color: &Rgba<u8>, img: &mut DynamicImage, pos: &Vec2, size: &Vec2) -> () {
    for x in 0..size.x {
        img.put_pixel(pos.x+x, pos.y, *color);
    }
    for y in 0..size.y {
        img.put_pixel(pos.x, pos.y+y, *color);
    }
}