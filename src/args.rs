use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser};
use image::Rgba;

use crate::quad;
use crate::utils::Vec2;

const VALUE_NAME_COLOR: &str = "COLOR";
const VALUE_NAME_IMAGE: &str = "IMAGE";
const ARG_GRP_IN: &str = "input_args";
const ARG_GRP_OUT: &str = "output_args";

/// Calculate and draw quads over images in various formats
#[derive(Parser)]
#[command(name = "Quadtree over Media")]
#[command(version, about, long_about = None)]
//
pub(super) struct QomCli {
    #[command(flatten)]
    pub io: IOArgs,

    #[command(flatten)]
    pub calc: QuadArgs,

    #[command(flatten)]
    pub image: DrawingArgs,

    /// Output verbosity, repeat for more verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

//TODO add custom parser for piping
#[derive(Args)]
#[command(group(ArgGroup::new(ARG_GRP_IN).required(true)))]
#[command(group(ArgGroup::new(ARG_GRP_OUT).required(true)))]
pub(super) struct IOArgs {
    /// Location of input media
    #[arg(long, short, value_parser, value_name = VALUE_NAME_IMAGE, group = ARG_GRP_IN)]
    pub input: Option<PathBuf>,

    /// the input is coming from a pipe. must be a string in the `WxHx(colordepth)`.
    /// for example `1920x1080xRGB`
    #[arg(long = "pipein", value_name="FORMAT DETAILS", group = ARG_GRP_IN)]
    pub input_pipe: Option<String>,

    /// Location of output media
    #[arg(long, short, value_parser, value_name = VALUE_NAME_IMAGE, group = ARG_GRP_OUT)]
    pub output: Option<PathBuf>,

    /// the output pipe colordepth details
    #[arg(long = "pipeout", value_name="FORMAT DETAILS", group = ARG_GRP_OUT)]
    pub output_pipe: Option<String>,

    /// Output image quality, lower quality = smaller files and vice versa.
    ///
    /// Supported only for PNG and JPEG
    #[arg(long, value_enum, default_value_t = ImgQuality::Default)]
    pub output_quality: ImgQuality,
}

#[derive(Args)]
pub(super) struct QuadArgs {
    /// Minimun number of iterations that will always be performed
    #[arg(long, value_parser, default_value_t = quad::DEFAULT_MIN_DEPTH)]
    pub min_depth: u8,

    /// Minimum allowed size of a quad. Accepts any two number `x` `y`
    /// separated by an ascii punctuation character, examples: `[23,12]` `{55;56}` `4-2` `007=6`
    #[arg(long, value_parser = parse_vec2, default_value_t = quad::DEFAULT_MIN_SIZE)]
    pub min_quad_size: Vec2,

    /// Maximum color difference between quadrants
    ///
    /// A quadrant is split if the color difference is above this
    ///     value ie: `MAX(avgcolor)-MIN(avgcolor) > threshold`.
    /// A value of `0000` will always split, a value of `FFFF` will never split;
    /// for example to split only on the Alpha channel use `FFF0`
    ///
    /// Passed as a valid CSS color.
    /// [default: rgba(10,10,10,255)]
    #[arg(long, short, value_parser = parse_color, value_name = VALUE_NAME_COLOR)]
    pub treshold: Option<Rgba<u8>>,
}

#[derive(Args)]
pub(super) struct DrawingArgs {
    /// Color of the lines defining the quads
    /// [default: "deeppink"]
    #[arg(long, short, value_parser = parse_color)]
    pub color: Option<Rgba<u8>>,

    /// When a new image is drawn this will be the default backround color
    #[arg(long, short, value_parser = parse_color, value_name = VALUE_NAME_COLOR)]
    pub background: Option<Rgba<u8>>,

    /// Create the OUTPUT without drawing over a copy of INPUT media
    #[arg(long, value_parser)]
    pub no_drawover: bool,

    /// Fill the quads with the relative average color value
    ///
    /// Implies --no-drawover.
    #[arg(long, value_parser)]
    pub fill: bool,

    /// Image used to fill the quads
    ///
    /// If `--fill` is also specified, it will multiply each pixel of this image
    /// by the average color of the quad
    #[arg(long, value_parser, value_name = VALUE_NAME_IMAGE)]
    pub fill_with: Option<PathBuf>,

    /// Draw the quad only if the average color is greater than this value
    #[arg(long, value_parser = parse_color, value_name = VALUE_NAME_COLOR)]
    pub filter_gt: Option<Rgba<u8>>,
    /// Draw the quad only if the average color is lesser than this value
    #[arg(long, value_parser = parse_color, value_name = VALUE_NAME_COLOR)]
    pub filter_lt: Option<Rgba<u8>>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum ImgQuality {
    /// Default image quality options
    Default,
    /// Optimize for size
    Min,
    /// Optimize for quality
    Max,
}

/// uses colorparser to parse the given color
pub(super) fn parse_color(s: &str) -> Result<Rgba<u8>, String> {
    match csscolorparser::parse(s) {
        Ok(c) => Ok(Rgba(c.to_rgba8())),
        Err(e) => Err(e.to_string()),
    }
}

/// parses vec2. Supported formats: `x,y`, `x;y`, `[x,y]`
pub(super) fn parse_vec2(s: &str) -> Result<Vec2, String> {
    let split: Vec<&str> = s
        .split(|c: char| c.is_ascii_punctuation())
        .filter(|&p| !p.is_empty())
        .collect();
    if split.len() > 2 {
        return Err("not a vec2".to_owned());
    }
    let x: u32 = match split[0].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err("not a valid number".to_owned()),
    };
    let y: u32 = match split[1].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err("not a valid number".to_owned()),
    };
    let v = Vec2 { x, y };
    if v < quad::DEFAULT_MIN_SIZE {
        Err("min quad size is too small".to_owned())
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::{utils::Vec2, *};

    #[test]
    fn parses_rgba() {
        assert_eq!(
            parse_color("rgb(250,251,252)").unwrap(),
            Rgba([250, 251, 252, 255])
        );
        assert_eq!(parse_color("#a1b2c3").unwrap(), Rgba([161, 178, 195, 255]));
        assert_eq!(parse_color("#ff00007f").unwrap(), Rgba([255, 0, 0, 127]));
        assert_eq!(parse_color("red").unwrap(), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn parses_vec2() {
        assert_eq!(parse_vec2("10,20"), Ok(Vec2 { x: 10, y: 20 }));
        assert_eq!(parse_vec2("-10-20-"), Ok(Vec2 { x: 10, y: 20 }));
        assert_eq!(parse_vec2("007=6"), Ok(Vec2 { x: 7, y: 6 }));
        assert_eq!(parse_vec2("(015,27)"), Ok(Vec2 { x: 15, y: 27 }));
        assert_eq!(parse_vec2("{264,664}"), Ok(Vec2 { x: 264, y: 664 }));
    }

    #[test]
    fn err_parsing_vec2() {
        assert_eq!(parse_vec2("10-11-12"), Err("not a vec2".to_owned()));
        assert_eq!(
            parse_vec2("2,2"),
            Err("min quad size is too small".to_owned())
        );
    }
}
