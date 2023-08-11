// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use assert_matches::assert_matches;

use semantic_release_cargo::{verify_conditions, verify_conditions_with_alternate};
// use semantic_release_cargo::Error;

#[test]
fn verify_simple_workspaces_is_ok() {
    set_registry_token();

    verify_workspace_is_ok(None, "basic");
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || verify_workspace_is_ok(None, "dependencies"),
    );
}

#[test]
fn verify_workspace_with_alternate_registry_is_ok() {
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || {
            with_env_var("CARGO_REGISTRIES_TEST_TOKEN", "fake_value", || {
                verify_workspace_is_ok(Some("test"), "dependencies_alternate_registry")
            })
        },
    );
}

#[test]
fn verify_workspace_with_cycle_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("cycle");

    let result = verify_conditions(io::sink(), Some(path));

    assert!(result.is_err());

    // assert_matches!(
    //     result,
    //     Err(Error::WorkspaceCycles {
    //         crate1: _,
    //         crate2: _,
    //     })
    // );
}

#[test]
fn verify_unknown_workspace_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("unknown");

    let result = verify_conditions(io::sink(), Some(&path));

    assert!(result.is_err());
    // assert_matches!(result, Err(Error::WorkspaceError(_)));
}

#[test]
fn verify_with_git_dependency_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("git_dep");

    let result = verify_conditions(io::sink(), Some(&path));

    assert!(result.is_err());
    // assert_matches!(
    //     result,
    //     Err(Error::BadDependency {
    //         from: _,
    //         to: _,
    //         typ: _,
    //     })
    // );
}

#[ignore]
#[test]
fn verify_with_git_and_version_dependency_is_ok() {
    with_env_var("CARGO_REGISTRY_TOKEN", "fake_token", || {
        let path = get_test_data_manifest_path("git_dep_version");

        let result = verify_conditions(io::sink(), Some(&path));

        assert_matches!(result, Ok(_));
    });
}

fn verify_workspace_is_ok(alternate_registry: Option<&str>, dir: impl AsRef<Path>) {
    let path = get_test_data_manifest_path(dir);

    let result = verify_conditions_with_alternate(io::sink(), alternate_registry, Some(&path));

    assert_matches!(result, Ok(_));
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

fn set_registry_token() {
    env::set_var("CARGO_REGISTRY_TOKEN", "fake_token");
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
