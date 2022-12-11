// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::{Path, PathBuf};

use assert_matches::assert_matches;

use semantic_release_cargo::{verify_conditions, Error};

#[test]
fn verify_without_env_var_is_error() {
    let path = get_test_data_manifest_path("basic");

    let result = verify_conditions(io::sink(), Some(&path));

    assert_matches!(result, Err(Error::VerifyError { reason: _ }));
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
