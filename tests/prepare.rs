// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use guppy::{graph::PackageGraph, MetadataCommand};
use semver::Version;
use tempfile::{tempdir, TempDir};
use toml_edit::{DocumentMut, Table};

use semantic_release_cargo::prepare;

#[test]
fn prepare_basic() {
    let (_tempdir, manifest) = copy_workspace("basic");

    prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

    let graph = get_package_graph(manifest);
    let workspace = graph.workspace();
    let pkg = workspace.member_by_path("").expect("Couldn't get root pkg");
    assert_eq!(pkg.version(), &Version::new(2, 0, 0));
}

#[test]
fn prepare_with_depedencies() {
    let (_tempdir, manifest) = copy_workspace("dependencies");

    prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

    let graph = get_package_graph(&manifest);
    for pkg in graph.workspace().iter() {
        assert_eq!(pkg.version(), &Version::new(2, 0, 0));
    }
    let cargo_toml = get_toml_document(&manifest);
    let root = cargo_toml.as_table();
    assert_eq!(get_dep_version(root, "dependencies", "dep1"), "2.0.0");
    assert_eq!(
        get_dep_version(root, "build-dependencies", "build1"),
        "2.0.0"
    );
}

#[test]
fn prepare_with_dependencies_with_explicit_version() {
    let (_tempdir, manifest) = copy_workspace("dependencies_with_explicit_version");

    prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

    let graph = get_package_graph(&manifest);
    for pkg in graph.workspace().iter() {
        assert_eq!(pkg.version(), &Version::new(2, 0, 0));
    }
    let cargo_toml = get_toml_document(&manifest);
    let root = cargo_toml.as_table();
    assert_eq!(get_dep_version(root, "dependencies", "dep1"), "2.0.0");
    assert_eq!(
        get_dep_version(root, "build-dependencies", "build1"),
        "2.0.0"
    );
    assert_eq!(get_dep_version(root, "dev-dependencies", "dev1"), "2.0.0")
}

#[test]
fn prepare_with_dependencies_with_aliased_package_names() {
    let (_tempdir, manifest) = copy_workspace("dependencies_with_aliased_pkg");

    prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

    let graph = get_package_graph(&manifest);
    for pkg in graph.workspace().iter() {
        assert_eq!(pkg.version(), &Version::new(2, 0, 0));
    }
    let cargo_toml = get_toml_document(&manifest);
    let root = cargo_toml.as_table();
    assert_eq!(get_dep_version(root, "dependencies", "dep_one"), "2.0.0");
    assert_eq!(
        get_dep_version(root, "build-dependencies", "build_one"),
        "2.0.0"
    );
}

#[test]
fn prepare_with_depedencies_from_alternate_registry() {
    with_env_var(
        "CARGO_REGISTRIES_TEST_INDEX",
        "https://github.com/rust-lang/crates.io-index",
        || {
            let (_tempdir, manifest) = copy_workspace("dependencies_alternate_registry");

            prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

            let graph = get_package_graph(&manifest);
            for pkg in graph.workspace().iter() {
                assert_eq!(pkg.version(), &Version::new(2, 0, 0));
            }
            let cargo_toml = get_toml_document(&manifest);
            let root = cargo_toml.as_table();
            assert_eq!(get_dep_version(root, "dependencies", "dep1"), "2.0.0");
            assert_eq!(
                get_dep_version(root, "build-dependencies", "build1"),
                "2.0.0"
            );
        },
    )
}

#[test]
fn prepare_with_target_dependency() {
    let (_tempdir, manifest) = copy_workspace("target_dep");

    prepare(io::sink(), Some(&manifest), "2.0.0".into()).expect("prepare failed");

    let graph = get_package_graph(&manifest);
    for pkg in graph.workspace().iter() {
        assert_eq!(pkg.version(), &Version::new(2, 0, 0));
    }
    let cargo_toml = get_toml_document(&manifest);
    let root = cargo_toml.as_table();
    let target = get_sub_table(root, "target");
    let cfg_unix = get_sub_table(target, "cfg(unix)");
    assert_eq!(get_dep_version(cfg_unix, "dependencies", "dep1"), "2.0.0");
}

fn copy_workspace(workspace: impl AsRef<Path>) -> (TempDir, PathBuf) {
    let workspace = workspace.as_ref();
    let tempdir = tempdir().expect("Couldn't create temp dir");
    let srcdir = get_workspace_dir(workspace);

    copy(srcdir, tempdir.path(), &CopyOptions::new()).expect("Couldn't copy the workspace");
    let mut cargo_toml = tempdir.path().join(workspace);
    cargo_toml.push("Cargo.toml");

    (tempdir, cargo_toml)
}

fn get_workspace_dir(workspace: impl AsRef<Path>) -> PathBuf {
    let mut path = PathBuf::from(file!());

    path.pop();
    path.pop();
    path.push("test_data");
    path.push(workspace);

    path
}

fn get_package_graph(manifest_path: impl Into<PathBuf>) -> PackageGraph {
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path(manifest_path)
        .build_graph()
        .expect("Couldn't build graph")
}

fn get_toml_document(path: impl AsRef<Path>) -> DocumentMut {
    let toml = fs::read_to_string(path).expect("Couldn't read file");
    toml.parse().expect("Couldn't parse toml file")
}

fn get_dep_version<'a>(table: &'a Table, dep_table: &str, dep: &str) -> &'a str {
    get_sub_table(table, dep_table)
        .get(dep)
        .unwrap_or_else(|| panic!("no {} dependency item", dep))
        .as_table_like()
        .unwrap_or_else(|| panic!("no {} dependency table-like", dep))
        .get("version")
        .expect("no version item")
        .as_value()
        .expect("no version value")
        .as_str()
        .expect("version not a string")
}

fn get_sub_table<'a>(table: &'a Table, sub: &str) -> &'a Table {
    table[sub]
        .as_table()
        .unwrap_or_else(|| panic!("no {} table", sub))
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
