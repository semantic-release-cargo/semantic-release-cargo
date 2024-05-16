// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

// use assert_matches::assert_matches;

use semantic_release_cargo::{verify_conditions, verify_conditions_with_alternate};
// use semantic_release_cargo::Error;

#[test]
fn verify_without_env_var_is_error() {
    let path = get_test_data_manifest_path("basic");

    let result = verify_conditions(Some(&path));

    assert!(result.is_err());
    // assert_matches!(result, Err(Error::VerifyError { reason: _ }));
}

#[test]
fn verify_alternate_registry_throws_error_if_correct_token_not_set() {
    let path = get_test_data_manifest_path("dependencies_alternate_registry");

    // fails if no registry is set.
    let result = verify_conditions_with_alternate(Some("test"), Some(&path));
    assert!(result.is_err());

    // fails if the wrong token is set.
    with_env_var("CARGO_REGISTRY_TOKEN", "fake_token", || {
        let result = verify_conditions_with_alternate(Some("test"), Some(&path));
        assert!(result.is_err());
    });
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
