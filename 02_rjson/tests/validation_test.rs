use rjson::{JsonError, JsonObject};
use rstest::*;
use std::env;
use std::path::{Path, PathBuf};

#[rstest]
fn files_pass(#[files("tests/files/pass*.json")] path: PathBuf) {
    let result = JsonObject::read_file(path.as_path().to_str().unwrap());
    assert_eq!(result.is_ok(), true);
}

#[rstest]
fn files_fail(#[files("tests/files/fail*.json")] path: PathBuf) {
    let result = JsonObject::read_file(path.as_path().to_str().unwrap());
    assert_eq!(result.is_err(), true);
}
