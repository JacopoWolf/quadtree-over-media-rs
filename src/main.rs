mod args;
mod drawing;
mod quad;
mod utils;

use log::{info, trace};
use simplelog::*;

use crate::args::*;
use crate::drawing::*;
use crate::quad::*;
use clap::Parser;
use image::{codecs::*, *};
use std::path::PathBuf;
use std::{fs::File, path::Path};

fn main() {
    let cli = QomCli::parse();

    SimpleLogger::init(
        match cli.verbose {
            0 => LevelFilter::Off,
            1 => LevelFilter::Info,
            (2..=u8::MAX) => LevelFilter::Trace,
        },
        Config::default(),
    )
    .unwrap();

    let keep_colors = cli.image.no_drawover || cli.image.fill || cli.image.fill_with.is_some();
    info!(
        "will {} keeping color averages",
        match keep_colors {
            true => "be",
            false => "not be",
        }
    );

    match cli.io.input {
        // file as arg
        Some(ref path) => {
            let img_in = match load_image(path) {
                Ok(img) => img,
                Err(error) => panic!("problem opening input file: {:?}", error),
            };
            let img_output = calculate_and_draw(&img_in, &cli.calc, &cli.image, keep_colors);
            output_image(&img_output, &cli.io);
            info!("... done!")
        }
        // pipe, loop std::in
        None => todo!(),
    }
}

fn calculate_and_draw(
    source: &DynamicImage,
    calc: &QuadArgs,
    draw: &DrawingArgs,
    keepcolors: bool,
) -> DynamicImage {
    info!("calculating...");
    let structure = calc_quads(
        &source,
        &calc.min_quad_size,
        calc.min_depth,
        &calc.treshold.unwrap_or(DEFAULT_TRESHOLD),
        keepcolors,
    );
    trace!("subdivided image into {} quads", structure.quads.len());
    info!("generating output image...");
    //BUG when only color and no-drawover are specified the pixel average color is still drawn
    if keepcolors {
        let img_fill_with: Option<DynamicImage> = match draw.fill_with {
            Some(ref path) => {
                trace!("loading image for fill-with");
                match load_image(path) {
                    Ok(img) => Some(img),
                    Err(error) => panic!("problem opening fill-with image: {:?}", error),
                }
            }
            None => None,
        };
        draw_quads(
            &structure,
            &draw.color,
            &draw.background,
            draw.fill,
            &img_fill_with,
            &gen_fill_range(draw),
        )
    } else {
        draw_quads_on(&source, &structure, &draw.color)
    }
}

fn load_image(source: &PathBuf) -> ImageResult<DynamicImage> {
    info!("loading {} ...", source.to_str().unwrap());
    image::io::Reader::open(&source)
        .expect("error while opening image")
        .with_guessed_format()
        .unwrap()
        .decode()
}

fn output_image(img: &DynamicImage, io: &IOArgs) {
    match io.output {
        // image file
        Some(ref path) => {
            info!("saving image to {} ...", path.to_str().unwrap());
            match save_image_fs(img, path, &io.output_quality) {
                Ok(_) => {}
                Err(error) => panic!("cannot save image: {:?}", error),
            }
        }
        // pipe output
        None => todo!(),
    }
}

fn save_image_fs(img: &DynamicImage, path: &PathBuf, quality: &ImgQuality) -> ImageResult<()> {
    match ImageFormat::from_path(path).expect("output is not a supported format!") {
        ImageFormat::Png => png::PngEncoder::new_with_quality(
            open_stream(path),
            match quality {
                ImgQuality::Default => png::CompressionType::Default,
                ImgQuality::Min => png::CompressionType::Fast,
                ImgQuality::Max => png::CompressionType::Best,
            },
            png::FilterType::Adaptive,
        )
        .write_image(
            &img.to_rgba8(),
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        ),
        ImageFormat::Jpeg => jpeg::JpegEncoder::new_with_quality(
            open_stream(path),
            match quality {
                ImgQuality::Default => 70,
                ImgQuality::Min => 30,
                ImgQuality::Max => 100,
            },
        )
        .write_image(
            &img.to_rgba8(),
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        ),
        _ => img.save(path),
    }
}

fn open_stream(path: &Path) -> File {
    File::create(path).expect("cannot open output file path")
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
