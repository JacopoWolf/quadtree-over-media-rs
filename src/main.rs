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
mod args;
mod drawing;
mod imageio;
mod quad;
mod utils;

use crate::args::*;
use crate::drawing::{apply_background_color, draw_quads, draw_quads_simple};
use crate::imageio::{load_image, save_image};
use crate::quad::*;
use clap::Parser;
use image::*;
use log::{info, trace};
use simplelog::*;
use std::collections::HashMap;

fn main() {
    let cli = QomCli::parse();

    SimpleLogger::init(
        match cli.verbose {
            0 => LevelFilter::Off,
            1 => LevelFilter::Info,
            (2..=u8::MAX) => LevelFilter::Trace,
        },
        ConfigBuilder::default()
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .build(),
    )
    .unwrap();

    // load source image to process
    let img_in = match load_image(&cli.io.input) {
        Ok(img) => img,
        Err(error) => panic!("problem opening input image: {error:?}"),
    };

    // load additional image
    let img_fill_with: Option<DynamicImage> = match cli.image.fill_with {
        Some(ref path) => match load_image(path) {
            Ok(img) => Some(if cli.image.background.is_some() {
                apply_background_color(&img, cli.image.background.as_ref().unwrap())
            } else {
                img
            }),
            Err(error) => panic!("problem opening fill-with image: {error:?}"),
        },
        None => None,
    };

    // process
    let img_output = calculate_and_draw(&img_in, &img_fill_with, &cli.calc, &cli.image);

    // save processed image
    info!("saving image to '{}' ...", cli.io.output.to_str().unwrap());
    match save_image(&img_output, &cli.io.output, &cli.io.output_quality) {
        Ok(_) => {}
        Err(error) => panic!("cannot save image: {error:?}"),
    }
    info!("... all done!")
}

fn calculate_and_draw(
    source: &DynamicImage,
    img_fill_with: &Option<DynamicImage>,
    calc: &QuadArgs,
    draw: &DrawingArgs,
) -> DynamicImage {
    info!("calculating...");
    let structure = calc_quads(
        source,
        &calc.min_quad_size,
        calc.min_depth,
        &calc.treshold.unwrap_or(DEFAULT_TRESHOLD),
        draw.fill,
    );
    trace!("subdivided image into {} quads", structure.map.len());
    info!("generating output image...");
    if draw.no_drawover || draw.fill || draw.fill_with.is_some() {
        let mut cache = HashMap::new();
        draw_quads(
            &structure,
            &draw.color,
            &draw.background,
            draw.fill,
            img_fill_with,
            &gen_fill_range(draw),
            &mut cache,
        )
    } else {
        draw_quads_simple(source, &structure, &draw.color)
    }
}

fn gen_fill_range(draw: &DrawingArgs) -> Option<[Rgba<u8>; 2]> {
    if draw.filter_lt.is_none() && draw.filter_gt.is_none() {
        None
    } else {
        Some([
            match draw.filter_lt {
                Some(c) => c,
                None => parse_color("0000").unwrap(),
            },
            match draw.filter_gt {
                Some(c) => c,
                None => parse_color("ffff").unwrap(),
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    //TODO write tests
}
