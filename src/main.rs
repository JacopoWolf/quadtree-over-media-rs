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
mod io;
mod quad;
mod utils;

use image::DynamicImage;
use crate::args::*;
use crate::drawing::{draw_quads, draw_quads_simple};
use crate::io::{load_image, save_image};
use crate::quad::*;
use crate::utils::Vec2;
use clap::Parser;
use core::panic;
use io::load_background;
use log::{debug, info, trace};
use simplelog::*;
use std::collections::HashMap;

fn main() {
    // initialization
    let cli = CliArgs::parse();

    SimpleLogger::init(
        match cli.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            (3..=u8::MAX) => LevelFilter::Trace,
        },
        ConfigBuilder::default()
            .set_time_level(LevelFilter::Off)
            .set_level_padding(LevelPadding::Right)
            .set_thread_level(LevelFilter::Off)
            .set_target_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Off)
            .build(),
    )
    .unwrap();

    // if input is a folder, check if output is also a folder
    if cli.io.input.is_dir() {
        if !cli.io.output.is_dir() {
            panic!("input is a directory but output isn't!");
        }

        multiple_images(&cli);
    } else {
        single_image(cli);
    }

    info!("DONE \\[T]/")
}

fn multiple_images(cli: &CliArgs) {
    let mut cache = HashMap::new();

    // load additional image
    let img_fill_with = load_background(&cli.image);

    // load all images into iterator
    let inputs = cli
        .io
        .input
        .read_dir()
        .expect("errors reading input directory")
        .filter(|dres| dres.as_ref().unwrap().path().is_file())
        .flatten();

    for entry in inputs {
        // load source image to process
        let img_in = match load_image(&entry.path()) {
            Ok(img) => img,
            Err(error) => panic!("problem opening input image: {error:?}"),
        };

        // process
        let img_out = calc_draw(&img_in, &img_fill_with, &cli.calc, &cli.image, &mut cache);

        // save processed image
        match save_image(
            &img_out,
            &cli.io.output.join(entry.file_name()),
            &cli.io.compression,
        ) {
            Ok(_) => {}
            Err(error) => panic!("cannot save image: {error:?}"),
        }
    }
}

fn single_image(cli: CliArgs) {
    // load source image to process
    let img_in = match load_image(&cli.io.input) {
        Ok(img) => img,
        Err(error) => panic!("problem opening input image: {error:?}"),
    };
    // load additional image
    let img_fill_with = load_background(&cli.image);

    // process
    let img_out = calc_draw(
        &img_in,
        &img_fill_with,
        &cli.calc,
        &cli.image,
        &mut HashMap::new(),
    );

    // save processed image
    match save_image(&img_out, &cli.io.output, &cli.io.compression) {
        Ok(_) => {}
        Err(error) => panic!("cannot save image: {error:?}"),
    }
}

fn calc_draw(
    source: &DynamicImage,
    img_fill_with: &Option<DynamicImage>,
    calc: &QuadArgs,
    draw: &DrawingArgs,
    cache: &mut HashMap<Vec2, DynamicImage>,
) -> DynamicImage {
    debug!("calculating...");
    let structure = calc_quads(
        source,
        &calc.min_quad_size,
        calc.min_depth,
        &calc.threshold.unwrap_or(DEFAULT_TRESHOLD),
        draw.fill,
    );
    trace!("subdivided image into {} quads", structure.map.len());
    info!("generating output image...");
    if draw.no_drawover || draw.fill || draw.fill_with.is_some() {
        draw_quads(
            &structure,
            &draw.color,
            &draw.background,
            draw.fill,
            img_fill_with,
            cache,
        )
    } else {
        draw_quads_simple(source, &structure, &draw.color)
    }
}
