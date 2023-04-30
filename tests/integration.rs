use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    process::{Command, Output},
};

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const RESOURCE_SQUARE: &str = "tests/square.png";
const BIN_PATH: &str = env!("CARGO_BIN_EXE_quadtree-over-media");
const TMP_DIR: &str = env!("CARGO_TARGET_TMPDIR");

const LNCH_ERR_MSG: &str = "Error launching the executable!";
const RUN_ERR_MSG: &str = "Error running the executable!";
const FILE_ERR_MSG: &str = "Error opening file!";

fn resource(name: &str) -> PathBuf {
    PathBuf::from(BASE_DIR).join(name)
}

fn strpath<'a>(pb: &'a PathBuf) -> &'a str {
    pb.to_str().expect("Error creating path!")
}

fn run(args: Vec<&str>) -> Output {
    Command::new(BIN_PATH)
        .args(args)
        .spawn()
        .expect(LNCH_ERR_MSG)
        .wait_with_output()
        .expect(RUN_ERR_MSG)
}

fn assert_images_eq(size: usize, a: &PathBuf, b: &PathBuf) {
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

#[test]
fn fails_no_arguments() {
    let status = run(vec![]).status;

    assert_eq!(status.success(), false);
    assert_eq!(status.code(), Some(2));
}

#[test]
fn simple() {
    let outp = PathBuf::from(TMP_DIR).join("simple.png");

    let output = run(vec![
        "-vv",
        "--color",
        "red",
        "--input",
        strpath(&resource(RESOURCE_SQUARE)),
        "--output",
        strpath(&outp),
    ]);

    assert_eq!(output.status.success(), true);
    assert_eq!(output.status.code(), Some(0));
    assert_images_eq(16 * 16, &outp, &resource("tests/expect/simple.png"))
}

#[test]
fn min_depth() {
    let outp = PathBuf::from(TMP_DIR).join("mindepth.png");

    let output = run(vec![
        "-vv",
        "--color",
        "red",
        "--input",
        strpath(&resource(RESOURCE_SQUARE)),
        "--output",
        strpath(&outp),
        "--min-depth",
        "0",
    ]);

    assert_eq!(output.status.success(), true);
    assert_eq!(output.status.code(), Some(0));
    assert_images_eq(16 * 16, &outp, &resource("tests/expect/mindepth.png"))
}
