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

use crate::args::*;
use crate::drawing::{draw_quads, draw_quads_squares, ImageCache};
use crate::io::*;
use crate::quad::*;
use clap::Parser;
use image::{DynamicImage, ImageError};
use log::{debug, error, info};
use simplelog::*;
use std::io::{Error, ErrorKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialization
    let cli = CliArgs::parse();

    // logging
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
    )?;

    if let 0 = check_rank(&cli.io)? {
        single_image(&cli)?
    } else {
        multiple_images(&cli)?
    }

    info!("DONE \\[T]/");
    Ok(())
}

fn check_rank(io: &IOArgs) -> Result<u8, Error> {
    if io.input.is_dir() {
        // folder
        if !io.output.is_dir() {
            error!("input is a directory but output isn't!");
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        Ok(u8::MAX)
    } else {
        // file
        if io.output.is_dir() {
            error!("input is a file, but output is a directory!");
            return Err(Error::from(ErrorKind::InvalidInput));
        }
        Ok(0)
    }
}

fn multiple_images(cli: &CliArgs) -> Result<(), ImageError> {
    let mut cache = ImageCache::new();

    // load additional image
    let img_fill_with = load_filler(&cli.image)?;

    // load all images into iterator
    let inputs = cli
        .io
        .input
        .read_dir()?
        .filter(|dres| dres.as_ref().unwrap().path().is_file())
        .flatten();

    for entry in inputs {
        // load source image to process
        let img_in = match load_image(&entry.path()) {
            Ok(img) => img,
            Err(error) => panic!("problem opening input image: {error:?}"),
        };

        // process
        let img_out =
            generate_quadtree_image(&img_in, &img_fill_with, &cli.calc, &cli.image, &mut cache);

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

    Ok(())
}

fn single_image(cli: &CliArgs) -> Result<(), ImageError> {
    // load source image to process
    let img_in = load_image(&cli.io.input)?;

    // load additional image
    let img_fill_with = load_filler(&cli.image)?;

    // process
    let img_out = generate_quadtree_image(
        &img_in,
        &img_fill_with,
        &cli.calc,
        &cli.image,
        &mut ImageCache::new(),
    );

    // save processed image
    save_image(&img_out, &cli.io.output, &cli.io.compression)?;

    Ok(())
}

fn generate_quadtree_image(
    source: &DynamicImage,
    img_fill_with: &Option<DynamicImage>,
    calc: &QuadArgs,
    draw: &DrawingArgs,
    cache: &mut ImageCache,
) -> DynamicImage {
    info!("calculating quads");

    let structure = calc_quads(
        source,
        &calc.min_quad_size,
        calc.min_depth,
        &calc.threshold.unwrap_or(DEFAULT_TRESHOLD),
        draw.fill,
    );

    debug!(
        "subdivided image into {} quads over {} recursions",
        structure.map.len(),
        structure.sizes.len() - 1
    );
    // if a new image has to be generated, recoloring needs to be applied or
    // if the filler image is not None, use the full version of the
    // drawing fn, otherwise simplify
    info!("generating output image");
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
        draw_quads_squares(source, &structure, &draw.color)
    }
}
