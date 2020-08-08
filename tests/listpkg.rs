// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    io::{BufRead, Cursor},
    path::{Path, PathBuf},
};

use semantic_release_rust::list_packages;

#[test]
fn list_basic_workspace() {
    let path = get_test_data_manifest_path("basic");
    let mut output = Vec::new();

    list_packages(Cursor::new(&mut output), Some(path)).expect("unable to list packages");

    let lines: Result<Vec<_>, _> = Cursor::new(&output).lines().collect();
    match lines {
        Ok(lines) => {
            assert!(lines[0].starts_with("basic"));
        }
        Err(_) => panic!("Unable to collect output lines"),
    }
}

#[test]
fn list_dependancies_workspace() {
    let path = get_test_data_manifest_path("dependencies");
    let mut output = Vec::new();

    list_packages(Cursor::new(&mut output), Some(path)).expect("unable to list packages");

    let lines: Result<Vec<_>, _> = Cursor::new(&output).lines().collect();
    match lines {
        Ok(lines) => {
            if lines[0].starts_with("build1") {
                assert!(lines[1].starts_with("dep1"));
            } else {
                assert!(lines[0].starts_with("dep1"));
                assert!(lines[1].starts_with("build1"));
            }
            assert!(lines[2].starts_with("dependencies"));
        }
        Err(_) => panic!("Unable to collect output lines"),
    }
}

fn get_test_data_manifest_path(dir: impl AsRef<Path>) -> PathBuf {
    let mut path = PathBuf::from(file!());

    path.pop();
    path.pop();
    path.push("test_data");
    path.push(dir);
    path.push("Cargo.toml");

    path
}
