// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    ffi::OsStr,
    io::{BufRead, Cursor},
    path::{Path, PathBuf},
};

use semantic_release_cargo::list_packages;

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
fn list_dependencies_workspace() {
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

#[test]
fn list_dependencies_with_alternate_registry_in_workspace() {
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || {
            let path = get_test_data_manifest_path("dependencies_alternate_registry");
            let mut output = Vec::new();

            list_packages(Cursor::new(&mut output), Some(path)).expect("unable to list packages");

            let lines: Result<Vec<_>, _> = Cursor::new(&output).lines().collect();
            match lines {
                Ok(lines) => {
                    if lines[0].starts_with("build1") {
                        assert!(lines[1].starts_with("dep1"), "{}", &lines.join("\n"));
                    } else {
                        assert!(lines[0].starts_with("dep1"));
                        assert!(lines[1].starts_with("build1"));
                    }
                    assert!(lines[2].starts_with("dependencies"));
                }
                Err(_) => panic!("Unable to collect output lines"),
            }
        },
    )
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

fn with_env_var<K, V, F>(key: K, value: V, f: F)
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    F: FnOnce(),
{
    use std::env;

    // Store the previous value of the var, if defined.
    let previous_val = env::var(key.as_ref()).ok();

    env::set_var(key.as_ref(), value.as_ref());
    (f)();

    // Reset or clear the var after the test.
    if let Some(previous_val) = previous_val {
        env::set_var(key.as_ref(), previous_val);
    } else {
        env::remove_var(key.as_ref());
    }
}
