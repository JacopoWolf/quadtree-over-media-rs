use std::collections::HashMap;

use image::*;
use crate::utils::*;
use crate::quad::*;

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
    depthsize_map: &HashMap<u8, Vec2>,
    border_color: &Option<Rgba<u8>>,
    background_color: &Option<Rgba<u8>>,
    quad_img: &Option<DynamicImage>,
    draw_range: &Option<[Rgba<u8>; 2]>,
) -> DynamicImage {
    let img_size = depthsize_map.get(&0).unwrap();

    let mut scaledimage_cache: HashMap<Vec2, DynamicImage> = HashMap::new();
    let mut img_out = DynamicImage::ImageRgba8(match background_color {
        Some(bgrc) => RgbaImage::from_pixel(img_size.x, img_size.y, *bgrc),
        None => RgbaImage::new(img_size.x, img_size.y),
    });
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
        let size = depthsize_map.get(&info.depth).unwrap();
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
                &mut scaledimage_cache,
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

fn color_between(color: &Option<Rgba<u8>>, from: &Rgba<u8>, to: &Rgba<u8>) -> bool {
    match color {
        Some(color) => (0..4).all(|i| color[i] >= from[i] && color[i] <= to[i]),
        None => false,
    }
}
