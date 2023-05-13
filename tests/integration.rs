mod common;

use crate::common::*;

#[test]
fn fails_no_arguments() {
    let status = run(vec![]).status;

    assert!(!status.success());
    assert_eq!(status.code(), Some(2));
}

#[test]
fn simple() {
    let outp = PathBuf::from(TMP_DIR).join("test.simple.png");

    let output = run(vec![
        "-vvv",
        "--color",
        "red",
        "--input",
        strpath(&resource(RES_SQUARE)),
        "--output",
        strpath(&outp),
    ]);

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    assert_images_eq(16 * 16, &outp, &resource(RES_EXP_SIMPLE))
}

#[test]
fn min_depth() {
    let outp = PathBuf::from(TMP_DIR).join("test.mindepth.png");

    let output = run(vec![
        "-vvv",
        "--color",
        "red",
        "--input",
        strpath(&resource(RES_SQUARE)),
        "--output",
        strpath(&outp),
        "--min-depth",
        "0",
    ]);

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
    assert_images_eq(16 * 16, &outp, &resource(RES_EXP_MINDEPTH))
}

#[test]
fn input_not_found() {
    let outp = PathBuf::from(TMP_DIR).join("shouldnt-exist.png");

    let output = run(vec![
        "-vvv",
        "--input",
        strpath(&resource("NOT-EXISTING.png")),
        "--output",
        strpath(&outp),
    ]);

    assert!(!outp.exists());
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(101));
}
