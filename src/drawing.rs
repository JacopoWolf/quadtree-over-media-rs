/* Copyright 2023 Comparin Jacopo
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::collections::HashMap;

use crate::quad::*;
use crate::utils::*;
use image::*;

//TODO unit test
/// create a copy of the original and draw quads outlines on it
pub fn draw_quads_simple(
    original: &DynamicImage,
    quads: &QuadStructure,
    color: &Option<Rgba<u8>>,
) -> DynamicImage {
    let mut copy_img = original.clone();

    for (pos, info) in quads.map.iter() {
        draw_square(
            &mut copy_img,
            pos,
            &quads.sizes[info.depth as usize],
            &color.unwrap_or(info.color.unwrap_or(DEFAULT_COLOR)),
            &None,
        );
    }
    copy_img
}

//TODO unit test
/// Draws quads based on the specified image and with the given args only if the color satisfies the filter
pub fn draw_quads(
    structure: &QuadStructure,
    border_color: &Option<Rgba<u8>>,
    background_color: &Option<Rgba<u8>>,
    multiply: bool,
    quad_img: &Option<DynamicImage>,
    draw_range: &Option<[Rgba<u8>; 2]>,
    cache: &mut HashMap<Vec2, DynamicImage>,
) -> DynamicImage {
    let img_size = structure.sizes[0];

    let mut img_out = DynamicImage::ImageRgba8(match background_color {
        Some(bgrc) => RgbaImage::from_pixel(img_size.x, img_size.y, *bgrc),
        None => RgbaImage::new(img_size.x, img_size.y), //transparent bg
    });
    for (pos, info) in structure.map.iter() {
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
            &structure.map,
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
                cache,
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

//TODO unit test
/// draw a square outline on the image
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

//TODO unit test
/// overlap an image on the specified position
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

//TODO remove in 2.0
fn is_color_between(color: &Option<Rgba<u8>>, from: &Rgba<u8>, to: &Rgba<u8>) -> bool {
    match color {
        Some(color) => (0..4).all(|i| color[i] >= from[i] && color[i] <= to[i]),
        None => false,
    }
}

fn multiply_image_by(src: &DynamicImage, by: &Rgba<u8>) -> DynamicImage {
    DynamicImage::ImageRgba8(
        RgbaImage::from_raw(
            src.width(),
            src.height(),
            src.pixels()
                .flat_map(|(_, _, p)| multiply_pixels(&p, by))
                .collect(),
        )
        .unwrap(),
    )
}

pub(super) fn apply_background_color(src: &DynamicImage, color: &Rgba<u8>) -> DynamicImage {
    DynamicImage::ImageRgba8(
        RgbaImage::from_raw(
            src.width(),
            src.height(),
            src.pixels()
                .flat_map(|(_, _, p)| match p[3] {
                    255 => p.0,
                    0 => color.0,
                    _ => multiply_pixels(&p, color),
                })
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
        ((a[3] as u16) * (b[3] as u16) / 255) as u8,
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

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    pub use test_case::test_case;

    #[test_case([096,096,096,096],[096,096,096,096],[036,036,036,036];"6666x6666")]
    #[test_case([096,096,096,096],[255,255,255,255],[096,096,096,096];"6666xFFFF")]
    #[test_case([011,255,022,238],[255,255,255,255],[011,255,022,238];"1F2ExFFFF")]
    #[test_case([011,255,022,238],[000,000,000,000],[000,000,000,000];"1F2Ex0000")]
    #[test_case([011,255,022,238],[136,136,136,136],[005,136,011,126];"1F2Ex8888")]
    fn multiplies_pixels(a: [u8; 4], b: [u8; 4], expects: [u8; 4]) {
        assert_eq!(multiply_pixels(&Rgba(a), &Rgba(b)), expects)
    }

    /* 0,0|AB e\|  X=(2,2)
     *    |CD f\|__ missing pixel
     *    |gh x\|
     *    |\\ \Z|
     */
    static TEST_QUADTREE: Lazy<HashMap<Vec2, QuadInfo>> = Lazy::new(|| {
        HashMap::from([
            (Vec2 { x: 0, y: 0 }, QuadInfo::new(1)), //A
            (Vec2 { x: 2, y: 0 }, QuadInfo::new(1)), //B
            (Vec2 { x: 0, y: 2 }, QuadInfo::new(1)), //C
            (Vec2 { x: 2, y: 2 }, QuadInfo::new(1)), //D
            (Vec2 { x: 5, y: 0 }, QuadInfo::new(1)), //e
            (Vec2 { x: 5, y: 2 }, QuadInfo::new(1)), //f
            (Vec2 { x: 0, y: 5 }, QuadInfo::new(1)), //g
            (Vec2 { x: 2, y: 5 }, QuadInfo::new(1)), //h
            (Vec2 { x: 5, y: 5 }, QuadInfo::new(1)), //x
            (Vec2 { x: 7, y: 7 }, QuadInfo::new(1)), //Z
        ])
    });
    const TEST_QUAD_SIZE: Vec2 = Vec2 { x: 2, y: 2 };

    #[test_case(Vec2{x:0,y:0},TEST_QUAD_SIZE; "none-rb")]
    #[test_case(Vec2{x:2,y:0},Vec2 { x: 3, y: 2 }; "expand-x")]
    #[test_case(Vec2{x:0,y:2},Vec2 { x: 2, y: 3 }; "expand-y")]
    #[test_case(Vec2{x:2,y:2},Vec2 { x: 3, y: 3 }; "expand-both")]
    #[test_case(Vec2{x:7,y:7},TEST_QUAD_SIZE; "at-bounds")]
    fn adjusts_size(pos: Vec2, expect_size: Vec2) {
        assert_eq!(
            adjust_quad_size(&pos, &TEST_QUAD_SIZE, &TEST_QUADTREE, &Vec2 { x: 9, y: 9 }),
            expect_size
        )
    }
}
