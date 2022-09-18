mod drawing;
mod args;
mod quad;
mod utils;

use crate::drawing::*;
use crate::args::*;
use crate::quad::*;
use clap::Parser;
use std::{fs::File, path::Path};

use image::{codecs::*, *};

fn main() {
    let args = QuadArgs::parse();
    //TODO implement video support
    _main_image(&args)
}

fn _main_image(args: &QuadArgs) {
    let img = load_image(&args.input);
    let keepcolors = args.no_drawover || args.fill || args.fill_with.is_some();
    println!("will keep averages");
    let (quadmap, sizemap) = calc_quads(
        &img,
        &args.min_quad_size,
        args.min_depth,
        &args.treshold.unwrap_or(DEFAULT_TRESHOLD),
        keepcolors,
    );
    println!("drawing...");
    let out = if keepcolors {
        draw_quads(
            &quadmap,
            &sizemap,
            &args.color,
            &args.background,
            args.fill,
            &(match &args.fill_with {
                Some(fimg) => Some(load_image(&fimg)),
                None => None,
            }),
            &gen_fill_range(&args),
        )
    } else {
        draw_quads_on(&img, &quadmap, &sizemap, &args.color)
    };
    println!("saving image to {}", args.output);
    save(&out, &args).expect("error while saving image");
    println!("... done!")
}

pub fn load_image(source: &String) -> DynamicImage {
    println!("loading {}", source);
    match image::io::Reader::open(&source)
        .expect("error while opening image")
        .with_guessed_format()
        .unwrap()
        .decode()
    {
        Ok(img) => {
            println!("image loaded successfully!");
            img
        }
        Err(_) => panic!("Problem decoding image:"),
    }
}

pub fn save(img: &DynamicImage, args: &QuadArgs) -> Result<(), ImageError> {
    let path = Path::new(&args.output);
    match ImageFormat::from_path(path).expect("output is not a supported format!") {
        ImageFormat::Png => png::PngEncoder::new_with_quality(
            stream(path),
            match args.output_quality {
                ImgOpt::Default => png::CompressionType::Default,
                ImgOpt::Min => png::CompressionType::Fast,
                ImgOpt::Max => png::CompressionType::Best,
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
            stream(path),
            match args.output_quality {
                ImgOpt::Default => 70,
                ImgOpt::Min => 30,
                ImgOpt::Max => 100,
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

fn stream(path: &Path) -> File {
    File::create(path).expect("cannot open output file path")
}

fn gen_fill_range(args: &QuadArgs) -> Option<[Rgba<u8>; 2]> {
    if args.filter_lt.is_none() && args.filter_gt.is_none() {
        None
    } else {
        Some([
            match args.filter_lt {
                Some(c) => c,
                None => parse_color("0000").unwrap(),
            },
            match args.filter_gt {
                Some(c) => c,
                None => parse_color("ffff").unwrap(),
            },
        ])
    }
}
