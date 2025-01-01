// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::{Path, PathBuf};

use assert_matches::assert_matches;

use semantic_release_cargo::verify_conditions_with_alternate;
// use semantic_release_cargo::Error;

#[test]
fn verify_simple_workspaces_with_cargo_toml_is_ok() {
    verify_workspace_is_ok(None, "basic_with_cargo_config")
}

fn verify_workspace_is_ok(alternate_registry: Option<&str>, dir: impl AsRef<Path>) {
    let previous_dir = std::env::current_dir().unwrap();
    let mut test_dir = PathBuf::from(file!());
    test_dir.pop();
    test_dir.pop();
    test_dir.push("test_data");
    test_dir.push(dir);

    // change into the testing directory
    std::env::set_current_dir(test_dir).unwrap();

    let manifest_path = PathBuf::from("Cargo.toml");

    let result = verify_conditions_with_alternate(alternate_registry, Some(&manifest_path));

    // revert to the original directory
    std::env::set_current_dir(previous_dir).unwrap();

    assert_matches!(result, Ok(_));
}
