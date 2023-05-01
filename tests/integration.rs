mod common;
use crate::common::*;

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
