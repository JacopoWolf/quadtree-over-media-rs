use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output},
};

// CONSTANTS

pub const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");
pub const BIN_PATH: &str = env!("CARGO_BIN_EXE_quadtree-over-media");
pub const TMP_DIR: &str = env!("CARGO_TARGET_TMPDIR");

// RESOURCES

pub const RES_SQUARE: &str = "square.png";
pub const RES_EXP_SIMPLE: &str = "expect/simple.png";
pub const RES_EXP_MINDEPTH: &str = "expect/mindepth.png";

// FUNCTIONS

pub fn resource(name: &str) -> PathBuf {
    PathBuf::from(BASE_DIR).join("tests").join(name)
}

pub fn strpath(pb: &Path) -> &str {
    pb.to_str().expect("Error creating path!")
}

pub fn run(args: Vec<&str>) -> Output {
    println!("arguments: {args:?}");
    Command::new(BIN_PATH)
        .args(args)
        .spawn()
        .expect("Error launching the executable!")
        .wait_with_output()
        .expect("Error running the executable!")
}

pub const FILE_ERR_MSG: &str = "Error opening file!";

pub fn assert_images_eq(size: usize, a: &PathBuf, b: &PathBuf) {
    let mut a_buf = Vec::with_capacity(size);
    let a_size = File::open(a)
        .expect(FILE_ERR_MSG)
        .read_to_end(&mut a_buf)
        .expect(FILE_ERR_MSG);
    let mut b_buf = Vec::with_capacity(size);
    let b_size = File::open(b)
        .expect(FILE_ERR_MSG)
        .read_to_end(&mut b_buf)
        .expect(FILE_ERR_MSG);
    assert_eq!(a_size, b_size);
    assert!(a_buf.eq(&b_buf))
}
