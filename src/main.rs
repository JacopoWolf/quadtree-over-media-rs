mod quad;
mod utils;

use clap::Parser;
use image::Rgba;

const DEFAULT_MIN_DEPTH: u32 = 4;
const DEFAULT_COLOR: Rgba<u8> = Rgba([255, 20, 147, 255]); //DeepPink

/// Calculate and draw quads over images, detecting "active" areas
/// and do nice stuff with that
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Location of input media. Can be any supported image
    #[clap(short, value_parser)]
    input: String,

    /// Location of output media
    #[clap(short, value_parser)]
    output: String,

    /// Minimun number of iterations
    #[clap(long, value_parser, default_value_t = DEFAULT_MIN_DEPTH)]
    min_depth: u32,

    /// The color of the lines defining the quads.
    #[clap(long, short, value_parser = parse_rgba)]
    color: Option<Rgba<u8>>,

    /// fill the quads with the relative average color value
    #[clap(long, value_parser)]
    fill: bool,

    /// create the OUTPUT without drawing over a copy of INPUT media
    #[clap(long, value_parser)]
    quads_only: bool,
}

fn main() {
    let args = Args::parse();
    draw_and_save(&args)
}

pub fn draw_and_save(args: &Args) {
    println!("loading {}", args.input);
    let img = match image::io::Reader::open(&args.input)
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
    };
    let img_out = quad::draw_quads_on_image(&img, &args);

    println!("saving image to {}", args.output);
    img_out
        .save(&args.output)
        .expect("error while saving image");
}

fn parse_rgba(s: &str) -> Result<Rgba<u8>, String> {
    match csscolorparser::parse(s) {
        Ok(c) => Ok(Rgba { 0: c.to_rgba8() }),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_works() {
        draw_and_save(&Args {
            input: "tests/src/shapes.png".to_owned(),
            output: "tests/out/shapes.png".to_owned(),
            color: Option::Some(DEFAULT_COLOR),
            min_depth: DEFAULT_MIN_DEPTH,
            quads_only: true,
            fill: false
        })
    }
    #[test]
    fn parses_rgb() {
        assert_eq!(
            parse_rgba("rgb(250,251,252)").unwrap(),
            Rgba([250, 251, 252, 255])
        )
    }
    #[test]
    fn parses_rgb_hex() {
        assert_eq!(parse_rgba("#a1b2c3").unwrap(), Rgba([161, 178, 195, 255]))
    }

    #[test]
    fn parses_rgba_hex() {
        assert_eq!(parse_rgba("#ff00007f").unwrap(), Rgba([255,0,0,127]) )
    }

    #[test]
    fn parses_colorname() {
        assert_eq!(parse_rgba("red").unwrap(), Rgba([255, 0, 0, 255]))
    }
}
