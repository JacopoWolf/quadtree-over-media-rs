use std::collections::HashMap;

use crate::quad::*;
use crate::utils::*;
use image::*;

/// create a copy of the original and draw quads outlines on it
pub fn draw_quads_on(
    original: &DynamicImage,
    quads: &QuadStructure,
    color: &Option<Rgba<u8>>,
) -> DynamicImage {
    let mut copy = original.clone();

    for (pos, info) in quads.quads.iter() {
        draw_square(
            &mut copy,
            pos,
            &quads.sizes[info.depth as usize],
            &color.unwrap_or(info.color.unwrap_or(DEFAULT_COLOR)),
            &None,
        );
    }
    copy
}

/// Draws quads based on the specified image and with the given args only if the color satisfies the filter
pub fn draw_quads(
    structure: &QuadStructure,
    border_color: &Option<Rgba<u8>>,
    background_color: &Option<Rgba<u8>>,
    multiply: bool,
    quad_img: &Option<DynamicImage>,
    draw_range: &Option<[Rgba<u8>; 2]>,
) -> DynamicImage {
    let img_size = structure.sizes[0];

    let mut scaledimage_cache: HashMap<Vec2, DynamicImage> = HashMap::new();
    let mut img_out = DynamicImage::ImageRgba8(match background_color {
        Some(bgrc) => RgbaImage::from_pixel(img_size.x, img_size.y, *bgrc),
        None => RgbaImage::new(img_size.x, img_size.y), //transparent bg
    });
    for (pos, info) in structure.quads.iter() {
        if draw_range.is_some()
            && !is_color_between(
                &info.color,
                &draw_range.unwrap()[0],
                &draw_range.unwrap()[1],
            )
        {
            continue;
        }

        let size_adj = adjust_quad_size(
            pos,
            &structure.sizes[info.depth as usize],
            &structure.quads,
            &img_size,
        );

        match quad_img {
            Some(qimg) => draw_image(
                &mut img_out,
                qimg,
                pos,
                &size_adj,
                border_color,
                match multiply {
                    true => &info.color,
                    false => &None,
                },
                &mut scaledimage_cache,
            ),
            None => {
                draw_square(
                    &mut img_out,
                    pos,
                    &size_adj,
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
) {
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
    multiply_color: &Option<Rgba<u8>>,
    cache: &mut HashMap<Vec2, DynamicImage>,
) {
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
    match multiply_color {
        Some(c) => img.copy_from(&multiply_image_by(draw, c), pos.x, pos.y),
        None => img.copy_from(draw, pos.x, pos.y),
    }
    .expect("Error while writing sub-image");
    match border_color {
        Some(c) => draw_square(img, pos, size, c, &None),
        None => (),
    }
}

pub(crate) fn is_color_between(color: &Option<Rgba<u8>>, from: &Rgba<u8>, to: &Rgba<u8>) -> bool {
    match color {
        Some(color) => (0..4).all(|i| color[i] >= from[i] && color[i] <= to[i]),
        None => false,
    }
}

fn multiply_image_by(src: &DynamicImage, by: &Rgba<u8>) -> DynamicImage {
    DynamicImage::ImageRgba8(
        RgbaImage::from_vec(
            src.width(),
            src.height(),
            src.pixels()
                .flat_map(|(_, _, p)| multiply_pixels(&p, by))
                .collect(),
        )
        .unwrap(),
    )
}
fn multiply_pixels(a: &Rgba<u8>, b: &Rgba<u8>) -> [u8; 4] {
    [
        ((a[0] as u16) * (b[0] as u16) / 255) as u8,
        ((a[1] as u16) * (b[1] as u16) / 255) as u8,
        ((a[2] as u16) * (b[2] as u16) / 255) as u8,
        a[3],
    ]
}

/// Increase the size by 1 ther's not a quad there next to this one;
/// this check avoids empty line artifacts caused by the modulo
/// while halfing odd numbers in the quad size
fn adjust_quad_size(
    pos: &Vec2,
    size: &Vec2,
    quadinf_map: &HashMap<Vec2, QuadInfo>,
    bounds: &Vec2,
) -> Vec2 {
    Vec2 {
        // find right
        x: if (pos.x + size.x) < bounds.x {
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
        y: if (pos.y + size.y) < bounds.y {
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
    }
}
