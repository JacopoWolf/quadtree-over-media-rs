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
use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser};
use image::Rgba;

use crate::quad;
use crate::utils::Vec2;

const VALUE_NAME_COLOR: &str = "COLOR";
const VALUE_NAME_IMAGE: &str = "IMAGE";
const ARG_GRP_IN: &str = "input_args";
const ARG_GRP_OUT: &str = "output_args";

#[derive(Parser)]
#[command(name = "Quadtree Over Media")]
#[command(version, about, long_about = None)]
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

#[derive(Args)]
#[command(group(ArgGroup::new(ARG_GRP_IN).required(true)))]
#[command(group(ArgGroup::new(ARG_GRP_OUT).required(true)))]
pub(super) struct IOArgs {
    /// Path to input media or media folder
    #[arg(long, short, value_parser, value_name = VALUE_NAME_IMAGE, group = ARG_GRP_IN)]
    pub input: PathBuf,

    /// Path to output media or target folder
    /// 
    /// Suggested formats are PNG, JPEG, and BMP
    #[arg(long, short, value_parser, value_name = VALUE_NAME_IMAGE, group = ARG_GRP_OUT)]
    pub output: PathBuf,

    /// Compression level of output image
    ///
    /// Supported only for PNG and JPEG
    #[arg(long, value_enum, default_value_t = ImgCompression::Default)]
    pub compression: ImgCompression,
}

#[derive(Args)]
pub(super) struct QuadArgs {
    /// Minimun number of iterations that will always be performed
    /// 
    /// Unless the minimum size is reached, always perform at least this number of iterations
    /// even if the average color values would not have the quads split
    #[arg(long, value_parser, default_value_t = quad::DEFAULT_MIN_DEPTH)]
    pub min_depth: u8,

    /// Minimum allowed size of a quad. 
    /// 
    /// Accepts any two number `x;y` separated by an ascii punctuation character.
    /// e.g.: `[23,12]` `{55;56}` `4-2` `007=6`
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
    pub threshold: Option<Rgba<u8>>,
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
    #[arg(long, short, value_parser)]
    pub fill: bool,

    /// Image used to fill the quads
    ///
    /// If `--fill` is also specified, it will multiply each pixel of this image
    /// by the average color of the quad
    #[arg(long, short = 'w', value_parser, value_name = VALUE_NAME_IMAGE)]
    pub fill_with: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub(crate) enum ImgCompression {
    /// Maximum compression
    Max,
    /// Optimize for size
    High,
    /// Default settings
    Default,
    /// Optimize for quality
    Low,
    /// No compression
    No,
}

/// uses colorparser to parse the given color
pub(super) fn parse_color(s: &str) -> Result<Rgba<u8>, String> {
    match csscolorparser::parse(s) {
        Ok(c) => Ok(Rgba(c.to_rgba8())),
        Err(e) => Err(e.to_string()),
    }
}

const ERR_NOT_VEC2: &str = "not a vec2";
const ERR_NAN: &str = "not a valid number";
const ERR_QUAD_TOO_SMALL: &str = "min quad size is too small";

/// parses vec2. Supported formats: `x,y`, `x;y`, `[x,y]`
pub(super) fn parse_vec2(s: &str) -> Result<Vec2, String> {
    let split: Vec<&str> = s
        .split(|c: char| c.is_ascii_punctuation())
        .filter(|&p| !p.is_empty())
        .collect();
    if split.len() != 2 {
        return Err(ERR_NOT_VEC2.to_owned());
    }
    let x: u32 = match split[0].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err(ERR_NAN.to_owned()),
    };
    let y: u32 = match split[1].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return Err(ERR_NAN.to_owned()),
    };
    let v = Vec2 { x, y };
    if v < quad::DEFAULT_MIN_SIZE {
        Err(ERR_QUAD_TOO_SMALL.to_owned())
    } else {
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    /* https://github.com/frondeus/test-case/wiki/Syntax#output-matcher */
    use super::*;
    use crate::utils::Vec2;
    use test_case::test_case;

    #[test_case("rgb(250,251,252)"  => Rgba([250, 251, 252, 255]); "rgb")]
    #[test_case("#a1b2c3"           => Rgba([161, 178, 195, 255]); "hex")]
    #[test_case("#ff00007f"         => Rgba([255, 0, 0, 127]); "hexa")]
    #[test_case("red"               => Rgba([255, 0, 0, 255]); "txt")]
    fn parses_rgba(color_str: &str) -> Rgba<u8> {
        parse_color(color_str).unwrap()
    }

    #[test_case("10,20"     => Vec2{x:10,y:20}; "c")] // c => comma
    #[test_case("-10,20-"   => Vec2{x:10,y:20}; "c-noise")]
    #[test_case("007=006"   => Vec2{x:07,y:06}; "equals")]
    #[test_case("(015,27)"  => Vec2{x:15,y:27}; "c-c")]
    #[test_case("[15,027]"  => Vec2{x:15,y:27}; "c-s")]
    #[test_case("{264,664}" => Vec2{x:264,y:664}; "c-g")]
    fn parses_vec2(vec_str: &str) -> Vec2 {
        parse_vec2(vec_str).unwrap()
    }

    #[test_case("£€1@4$%"   => ERR_NAN; "nan-a")]
    #[test_case("a-a"       => ERR_NAN; "nan-b")]
    #[test_case("42"        => ERR_NOT_VEC2; "err-not-vec2-a")]
    #[test_case("10-11-12"  => ERR_NOT_VEC2; "err-not-vec2-b")]
    #[test_case("2,2"       => ERR_QUAD_TOO_SMALL; "err-too-small")]
    fn parses_vec2_err(vec_str: &str) -> String {
        parse_vec2(vec_str).unwrap_err()
    }
}
