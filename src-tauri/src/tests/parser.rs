use std::fs;

use crate::{
    dao::session::Session,
    parser::{FitParser, errors::ParseFitFileError},
};

#[test]
fn missing_file_fails_to_open() {
    let res = FitParser::from_file("/nonexistent/path/activity.fit");
    assert!(matches!(res, Err(ParseFitFileError::FileOpening(..))));
}

#[test]
fn garbage_file_fails_to_parse() {
    let path = std::env::temp_dir().join(format!("gtrs-test-garbage-{}.fit", std::process::id()));
    fs::write(&path, b"this is definitely not a FIT file, just some random bytes").unwrap();

    let res = FitParser::from_file(&path).and_then(Session::try_from);
    let _ = fs::remove_file(&path);

    assert!(matches!(res, Err(ParseFitFileError::FileReading(..))));
}
