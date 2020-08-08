// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use assert_matches::assert_matches;

use semantic_release_rust::{verify_conditions, Error};

#[test]
fn verify_simple_workspaces_is_ok() {
    set_registry_token();

    verify_workspace_is_ok("basic");
    verify_workspace_is_ok("dependencies");
}

#[test]
fn verify_workspace_with_cycle_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("cycle");

    let result = verify_conditions(io::sink(), Some(path));

    assert_matches!(result, Err(Error::WorkspaceCycles { crate1: _, crate2: _}));
}

#[test]
fn verify_unknown_workspace_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("unknown");

    let result = verify_conditions(io::sink(), Some(&path));

    assert_matches!(result, Err(Error::WorkspaceError(_)));
}

#[test]
fn verify_with_git_dependancy_is_error() {
    set_registry_token();
    let path = get_test_data_manifest_path("git_dep");

    let result = verify_conditions(io::sink(), Some(&path));

    assert_matches!(
        result,
        Err(Error::BadDependancy {
            from: _,
            to: _,
            typ: _,
        })
    );
}

#[test]
fn verify_with_git_and_version_dependancy_is_ok() {
    set_registry_token();
    let path = get_test_data_manifest_path("git_dep_version");

    let result = verify_conditions(io::sink(), Some(&path));

    assert_matches!(result, Ok(_));
}

fn verify_workspace_is_ok(dir: impl AsRef<Path>) {
    let path = get_test_data_manifest_path(dir);

    let result = verify_conditions(io::sink(), Some(&path));

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
