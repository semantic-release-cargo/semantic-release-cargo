// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;

use semantic_release_cargo::Error;

#[test]
fn error_format_includes_stderr() {
    // Create a path
    let path = PathBuf::from("Cargo.toml");

    // Create an error with stderr message by using the public constructor
    let error = Error::CargoPublishStatus {
        status: dummy_exit_status(),
        manifest_path: path,
        stderr: "Error: failed to publish\nCause: package already exists on registry".to_string(),
    };

    // Check that the formatted error message includes the stderr content
    let error_string = format!("{}", error);
    assert!(
        error_string.contains("failed to publish"),
        "Error should contain stderr content"
    );
    assert!(
        error_string.contains("package already exists"),
        "Error should contain all stderr content"
    );
}

fn dummy_exit_status() -> std::process::ExitStatus {
    use std::process::Command;

    // Create a real exit status with code 101
    if cfg!(windows) {
        Command::new("cmd").args(["/C", "exit 101"]).status()
    } else {
        Command::new("sh").args(["-c", "exit 101"]).status()
    }
    .expect("Failed to execute command")
}
