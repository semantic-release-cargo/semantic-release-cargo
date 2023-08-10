// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of the sementic release steps to for integraing a cargo-based Rust
//! project.

#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]

use std::{
    collections::HashMap,
    env, fmt, fs,
    io::{BufRead, Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
    result,
};

use guppy::{
    graph::{DependencyDirection, PackageGraph, PackageLink, PackageMetadata, PackageSource},
    MetadataCommand, PackageId,
};
use itertools::Itertools;
use log::{debug, error, info, log, trace, Level};
use serde::Serialize;
use toml_edit::{Document, InlineTable, Item, Table, Value};
use url::Url;

#[cfg(feature = "napi-rs")]
use napi_derive::napi;

mod error;

pub use error::{CargoTomlError, Error, Result};

/// Verify that the conditions for a release are satisfied.
///
/// The conditions for a release checked by this function are:
///
///    1. That the CARGO_REGISTRY_TOKEN environment variable is set and is
///       non-empty.
///    2. That it can construct the graph of all of the dependencies in the
///       workspace.
///    3. That the dependencies and build-dependencies of all of crates in the
///       workspace are suitable for publishing to `crates.io`.
///
/// If `manifest_path` is provided then it is expect to give the path to the
/// `Cargo.toml` file for the root of the workspace. If `manifest_path` is `None`
/// then `verify_conditions` will look for the root of the workspace in a
/// `Cargo.toml` file in the current directory. If one of the conditions for a
/// release are not satisfied then an explination for that will be written to
/// `output`.
///
/// This implments the `verifyConditions` step for `sementic-release` for a
/// Cargo-based rust workspace.
#[cfg(feature = "napi-rs")]
#[napi]
pub fn verify_conditions() -> Result<()> {
    let output = std::io::stdout();
    let manifest_path: Option<&Path> = None;
    internal_verify_conditions(output, manifest_path)
}

/// Verify that the conditions for a release are satisfied.
///
/// The conditions for a release checked by this function are:
///
///    1. That the CARGO_REGISTRY_TOKEN environment variable is set and is
///       non-empty.
///    2. That it can construct the graph of all of the dependencies in the
///       workspace.
///    3. That the dependencies and build-dependencies of all of crates in the
///       workspace are suitable for publishing to `crates.io`.
///
/// If `manifest_path` is provided then it is expect to give the path to the
/// `Cargo.toml` file for the root of the workspace. If `manifest_path` is `None`
/// then `verify_conditions` will look for the root of the workspace in a
/// `Cargo.toml` file in the current directory. If one of the conditions for a
/// release are not satisfied then an explination for that will be written to
/// `output`.
///
/// This implments the `verifyConditions` step for `sementic-release` for a
/// Cargo-based rust workspace.
#[cfg(not(feature = "napi-rs"))]
pub fn verify_conditions(
    output: impl Write,
    manifest_path: Option<impl AsRef<Path>>,
) -> Result<()> {
    internal_verify_conditions(output, manifest_path)
}

fn internal_verify_conditions(
    mut output: impl Write,
    manifest_path: Option<impl AsRef<Path>>,
) -> Result<()> {
    info!("Checking CARGO_REGISTRY_TOKEN");
    env::var_os("CARGO_REGISTRY_TOKEN")
        .and_then(|val| if val.is_empty() { None } else { Some(()) })
        .ok_or_else(|| {
            writeln!(output, "CARGO_REGISTRY_TOKEN empty or not set.")
                .map_err(Error::output_error)
                .and_then::<(), _>(|()| {
                    Err(Error::verify_error("CARGO_REGISTRY_TOKEN empty or not set"))
                })
                .unwrap_err()
        })?;

    info!("Checking that workspace dependencies graph is buildable");
    let graph = match get_package_graph(manifest_path) {
        Ok(graph) => graph,
        Err(err) => {
            return writeln!(
                output,
                "Unable to build workspace dependencies graph: {}",
                err
            )
            .map_err(|io_error| -> anyhow::Error { Error::output_error(io_error).into() })
            .and(Err(err));
        }
    };

    info!("Checking that the workspace does not contain any cycles");
    if let Some(cycle) = graph.cycles().all_cycles().next() {
        assert!(cycle.len() >= 2);
        let crate0 = get_crate_name(&graph, cycle[0]);
        let crate1 = get_crate_name(&graph, cycle[1]);
        return writeln!(
            output,
            "Workspace contains a cycle that includes (at least) {} and {}",
            crate0, crate1
        )
        .map_err(|io_error| -> anyhow::Error { Error::output_error(io_error).into() })
        .and_then(|()| -> Result<()> {
            Err(Error::WorkspaceCycles {
                crate1: crate0.to_owned(),
                crate2: crate1.to_owned(),
            }
            .into())
        });
    }

    info!("Checking that dependencies are suitable for publishing");
    for (from, links) in graph
        .workspace()
        .iter()
        .flat_map(|package| package.direct_links())
        .filter(|link| !link_is_publishable(link))
        .group_by(PackageLink::from)
        .into_iter()
    {
        debug!("Checking links for package {}", from.name());
        let cargo = read_cargo_toml(from.manifest_path().as_std_path())?;
        for link in links {
            if link.normal().is_present() {
                dependency_has_version(&cargo, &link, DependencyType::Normal).map_err(|err| {
                    writeln!(
                        output,
                        "Dependency {0} of {1} makes {1} not publishable.",
                        link.to().name(),
                        link.from().name()
                    )
                    .map_err(|io_error| -> anyhow::Error { Error::output_error(io_error).into() })
                    .and::<()>(Err(err))
                    .unwrap_err()
                })?;
            }
            if link.build().is_present() {
                dependency_has_version(&cargo, &link, DependencyType::Build).map_err(|err| {
                    writeln!(
                        output,
                        "Build dependency {0} of {1} makes {1} not publishable.",
                        link.to().name(),
                        link.from().name()
                    )
                    .map_err(|io_error| -> anyhow::Error { Error::output_error(io_error).into() })
                    .and::<()>(Err(err))
                    .unwrap_err()
                })?;
            }
        }
    }

    Ok(())
}

/// Prepare the Rust workspace for a release.
///
/// Preparing the release updates the version of each crate in the workspace and of
/// the intra-workspace dependencies. The `version` field in the `packages` table of
/// each `Cargo.toml` file in the workspace is set to the supplied version. The
/// `version` field of each dependency or build-dependency that is otherwise
/// identified by a workspace-relative path dependencies is also set to the supplied
/// version (the version filed will be added if it isn't already present).
///
/// This implments the `prepare` step for `sementic-release` for a Cargo-based Rust
/// workspace.
#[cfg(feature = "napi-rs")]
#[napi]
pub fn prepare(next_release_version: String) -> Result<()> {
    let output = std::io::stdout();
    let manifest_path: Option<&Path> = None;
    internal_prepare(output, manifest_path, next_release_version)
}

/// Prepare the Rust workspace for a release.
///
/// Preparing the release updates the version of each crate in the workspace and of
/// the intra-workspace dependencies. The `version` field in the `packages` table of
/// each `Cargo.toml` file in the workspace is set to the supplied version. The
/// `version` field of each dependency or build-dependency that is otherwise
/// identified by a workspace-relative path dependencies is also set to the supplied
/// version (the version filed will be added if it isn't already present).
///
/// This implments the `prepare` step for `sementic-release` for a Cargo-based Rust
/// workspace.
#[cfg(not(feature = "napi-rs"))]
pub fn prepare(
    output: impl Write,
    manifest_path: Option<&Path>,
    next_release_version: String,
) -> Result<()> {
    internal_prepare(output, manifest_path, next_release_version)
}

fn internal_prepare(
    _output: impl Write,
    manifest_path: Option<&Path>,
    next_release_version: String,
) -> Result<()> {
    info!("Building package graph");
    let graph = get_package_graph(manifest_path)?;

    let link_map = graph
        .workspace()
        .iter()
        .flat_map(|package| package.direct_links())
        .filter(|link| !link.dev_only() && link.to().in_workspace())
        .map(|link| (link.from().id(), link))
        .into_group_map();

    info!("Setting version information for packages in the workspace.");
    for package in graph.workspace().iter() {
        let path = package.manifest_path();
        debug!("reading {}", path.as_str());
        let mut cargo = read_cargo_toml(path.as_std_path())?;

        debug!("Setting the version for {}", package.name());
        set_package_version(&mut cargo, &next_release_version)
            .map_err(|err| err.into_error(path))?;

        if let Some(links) = link_map.get(package.id()) {
            debug!(
                "Setting the version for the dependencies of {}",
                package.name()
            );

            for link in links {
                if link.normal().is_present() {
                    set_dependencies_version(
                        &mut cargo,
                        &next_release_version,
                        DependencyType::Normal,
                        link.to().name(),
                    )
                    .map_err(|err| err.into_error(path))?;
                }
                if link.build().is_present() {
                    set_dependencies_version(
                        &mut cargo,
                        &next_release_version,
                        DependencyType::Build,
                        link.to().name(),
                    )
                    .map_err(|err| err.into_error(path))?;
                }
            }
        }

        debug!("writing {}", path.as_str());
        write_cargo_toml(path.as_std_path(), cargo)?;

        // Update the lockfile metadata.
        //
        // This code currently only updates the version number of the crate's
        // self-describing metadata.
        //
        // Unsupported: updating metadata of in-workspace dependencies. I
        // didn't take a stab at this yet because I don't have this issue
        // personall yet, and without a repository in which I can reproduce
        // this problem I think it's most responsible to keep the code simple
        // and readable.
        let lockfile_path = get_cargo_lock(path.as_std_path());
        if lockfile_path.exists() {
            debug!("reading {}", lockfile_path.to_string_lossy());
            let mut lockfile = read_cargo_toml(&lockfile_path)?;

            set_lockfile_self_describing_metadata(
                &mut lockfile,
                &next_release_version,
                package.name(),
            )?;

            debug!("writing {}", lockfile_path.to_string_lossy());
            write_cargo_toml(&lockfile_path, lockfile)?;
        }
    }

    Ok(())
}

#[cfg_attr(feature = "napi-rs", napi(object))]
#[derive(Debug, Default)]
/// Arguments to be passed to the `publish` function.
pub struct PublishArgs {
    /// Whether the `--no-dirty` flag should be passed to `cargo publish`.
    pub no_dirty: Option<bool>,

    /// A map of packages and features to pass to `cargo publish`.
    pub features: Option<HashMap<String, Vec<String>>>,

    /// Optionally passes a `--registry` flag `cargo publish`.
    pub registry: Option<String>,
}

/// Publish the publishable crates from the workspace.
///
/// The publishable crates are the crates in the workspace other than those
/// whose `package.publish` field is set to `false` or that includes a registry other
/// than `crates.io`.
///
/// This implments the `publish` step for `sementic-release` for a Cargo-based
/// Rust workspace.
#[cfg(feature = "napi-rs")]
#[napi]
pub fn publish(opts: Option<PublishArgs>) -> Result<()> {
    let output = std::io::stdout();
    let manifest_path: Option<&Path> = None;
    internal_publish(output, manifest_path, &opts.unwrap_or_default())
}

/// Publish the publishable crates from the workspace.
///
/// The publishable crates are the crates in the workspace other than those
/// whose `package.publish` field is set to `false` or that includes a registry other
/// than `crates.io`.
///
/// This implments the `publish` step for `sementic-release` for a Cargo-based
/// Rust workspace.
#[cfg(not(feature = "napi-rs"))]
pub fn publish(output: impl Write, manifest_path: Option<&Path>, opts: &PublishArgs) -> Result<()> {
    internal_publish(output, manifest_path, opts)
}

fn internal_publish(
    output: impl Write,
    manifest_path: Option<&Path>,
    opts: &PublishArgs,
) -> Result<()> {
    info!("getting the package graph");
    let graph = get_package_graph(manifest_path)?;

    let mut count = 0;
    let mut last_id = None;

    process_publishable_packages(&graph, |pkg| {
        count += 1;
        last_id = Some(pkg.id().clone());
        publish_package(pkg, opts)
    })?;

    let main_crate = match graph.workspace().member_by_path("") {
        Ok(pkg) if package_is_publishable(&pkg) => Some(pkg.name()),
        _ => last_id.map(|id| {
            graph
                .metadata(&id)
                .expect("id of a processed package not found in the package graph")
                .name()
        }),
    };

    if let Some(main_crate) = main_crate {
        debug!("printing release record with main crate: {}", main_crate);
        let name = format!("crate.io packages ({} packages published)", count);
        serde_json::to_writer(output, &Release::new(name, main_crate)?)
            .map_err(|err| Error::write_release_error(err, main_crate))?;
    } else {
        debug!("no release record to print");
    }

    Ok(())
}

/// List the packages from the workspace in the order of their dependencies.
///
/// The list of pacakges will be written to `output`. If `manifest_path` is provided
/// then it is expected to give the path to the `Cargo.toml` file for the root of the
/// workspace. If `manifest_path` is `None` then `list_packages` will look for the
/// root of the workspace in a `Cargo.toml` file in the current directory.
///
/// This is a debuging aid and does not directly correspond to a sementic release
/// step.
pub fn list_packages(
    #[cfg(not(feature = "napi-rs"))] mut output: impl Write,
    manifest_path: Option<impl AsRef<Path>>,
) -> Result<()> {
    #[cfg(feature = "napi-rs")]
    let mut output = std::io::stdout();

    info!("Building package graph");
    let graph = get_package_graph(manifest_path)?;

    process_publishable_packages(&graph, |pkg| {
        writeln!(output, "{}({})", pkg.name(), pkg.version()).map_err(Error::output_error)?;

        Ok(())
    })
}

fn get_package_graph(manifest_path: Option<impl AsRef<Path>>) -> Result<PackageGraph> {
    let manifest_path = manifest_path.as_ref().map(|path| path.as_ref());

    let mut command = MetadataCommand::new();
    if let Some(path) = manifest_path {
        command.manifest_path(path);
    }

    debug!("manifest_path: {:?}", manifest_path);

    command.build_graph().map_err(|err| {
        let path = match manifest_path {
            Some(path) => path.to_path_buf(),
            None => env::current_dir()
                .map(|path| path.join("Cargo.toml"))
                .unwrap_or_else(|e| {
                    error!("Unable to get current directory: {}", e);
                    PathBuf::from("unknown manifest")
                }),
        };
        Error::workspace_error(err, path).into()
    })
}

/// Is the source of the target of a dependencies publishable?
///
/// The target of a dependencies must be available on `crates.io` for the depending
/// package to be publishable. Workspace relative path dependencies will be published
/// before their depended on crates and the dependencies in the depended on crate
/// will have their `version` adjusted so those dependencies will be on `crates.io`
/// by the time the depended on crate is published.
fn target_source_is_publishable(source: PackageSource) -> bool {
    source.is_workspace() || source.is_crates_io()
}

/// Will this link prevent the `link.from()` package from being published.
///
/// `dev-dependencies` links will not prevent publication. For all other links the
/// target of the link must be either already on `crates.io` or it must be a
/// workspace relative path dependency (which will be published first).
fn link_is_publishable(link: &PackageLink) -> bool {
    let result = link.dev_only() || target_source_is_publishable(link.to().source());
    if result {
        trace!(
            "Link from {} to {} is publishable.",
            link.from().name(),
            link.to().name()
        );
    }

    result
}

/// Is a particular package publishable.
///
/// A package is publishable if either publication is unrestricted or the one
/// and only registry it is allowed to be published to is "crates.io".
fn package_is_publishable(pkg: &PackageMetadata) -> bool {
    let result = match pkg.publish() {
        guppy::graph::PackagePublish::Unrestricted => true,
        guppy::graph::PackagePublish::Registries(registries) => {
            registries.len() == 1 && registries[0] == "crates.io"
        }
        _ => todo!(),
    };

    if result {
        trace!("package {} is publishable", pkg.name());
    }

    result
}

fn process_publishable_packages<F>(graph: &PackageGraph, mut f: F) -> Result<()>
where
    F: FnMut(&PackageMetadata) -> Result<()>,
{
    info!("iterating the workspace crates in dependency order");
    for pkg in graph
        .query_workspace()
        .resolve_with_fn(|_, link| !link.dev_only())
        .packages(DependencyDirection::Reverse)
        .filter(|pkg| pkg.in_workspace() && package_is_publishable(pkg))
    {
        f(&pkg)?;
    }

    Ok(())
}

// Panics if id is not from graph
fn get_crate_name<'a>(graph: &'a PackageGraph, id: &PackageId) -> &'a str {
    graph
        .metadata(id)
        .unwrap_or_else(|_| panic!("id {} was not found in the graph {:?}", id, graph))
        .name()
}

fn publish_package(pkg: &PackageMetadata, opts: &PublishArgs) -> Result<()> {
    debug!("publishing package {}", pkg.name());

    let cargo = env::var("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("cargo"));

    let mut command = Command::new(cargo);
    command
        .args(["publish", "--manifest-path"])
        .arg(pkg.manifest_path());
    if !opts.no_dirty.unwrap_or_default() {
        command.arg("--allow-dirty");
    }
    if let Some(features) = opts.features.as_ref().and_then(|f| f.get(pkg.name())) {
        command.arg("--features");
        command.args(features);
    }
    if let Some(registry) = opts.registry.as_ref() {
        command.arg("--registry");
        command.arg(registry);
    }

    trace!("running: {:?}", command);

    let output = command
        .output()
        .map_err(|err| Error::cargo_publish(err, pkg.manifest_path().as_std_path()))?;

    let level = if output.status.success() {
        Level::Trace
    } else {
        Level::Info
    };

    trace!("cargo publish stdout");
    trace!("--------------------");
    log_bytes(Level::Trace, &output.stdout);

    log!(level, "cargo publish stderr");
    log!(level, "--------------------");
    log_bytes(level, &output.stderr);

    if output.status.success() {
        Ok(())
    } else {
        error!(
            "publishing package {} failed: {}",
            pkg.name(),
            output.status
        );
        Err(Error::cargo_publish_status(output.status, pkg.manifest_path().as_std_path()).into())
    }
}

fn log_bytes(level: Level, bytes: &[u8]) {
    let mut buffer = Cursor::new(bytes);
    let mut string = String::new();

    while let Ok(size) = buffer.read_line(&mut string) {
        if size == 0 {
            return;
        }
        log!(level, "{}", string);
        string.clear();
    }
}

/// Given the path to a cargo manifest, return the path to the associated
/// lock file. This function does not test the existence of the lockfile.
fn get_cargo_lock(path: &Path) -> PathBuf {
    path.parent().unwrap().join("Cargo.lock")
}

fn read_cargo_toml(path: &Path) -> Result<Document> {
    fs::read_to_string(path)
        .map_err(|err| Error::file_read_error(err, path))?
        .parse()
        .map_err(|err| Error::toml_error(err, path).into())
}

fn write_cargo_toml(path: &Path, cargo: Document) -> Result<()> {
    fs::write(path, cargo.to_string()).map_err(|err| Error::file_write_error(err, path).into())
}

fn get_top_table<'a>(doc: &'a Document, key: &str) -> Option<&'a Table> {
    doc.as_table().get(key).and_then(Item::as_table)
}

fn get_top_table_mut<'a>(doc: &'a mut Document, key: &str) -> Option<&'a mut Table> {
    doc.get_key_value_mut(key)
        .and_then(|(_key, value)| value.as_table_mut())
}

fn table_add_or_update_value(table: &mut Table, key: &str, value: Value) -> Option<()> {
    let entry = table.entry(key);

    match entry {
        toml_edit::Entry::Occupied(mut val) => {
            val.insert(Item::Value(value));
            Some(())
        }
        toml_edit::Entry::Vacant(val) => {
            val.insert(Item::Value(value));
            Some(())
        }
    }
}

fn inline_table_add_or_update_value(table: &mut InlineTable, key: &str, value: Value) {
    match table.get_mut(key) {
        Some(ver) => *ver = value,
        None => {
            table.get_or_insert(key, value);
        }
    }
}

fn dependency_has_version(doc: &Document, link: &PackageLink, typ: DependencyType) -> Result<()> {
    let top_key = match typ {
        DependencyType::Normal => "dependencies",
        DependencyType::Build => "build-dependencies",
    };

    trace!(
        "Checking for version key for {} in {} section of {}",
        link.to().name(),
        top_key,
        link.from().name()
    );
    get_top_table(doc, top_key)
        .and_then(|deps| deps.get(link.to().name()))
        .and_then(Item::as_table_like)
        .and_then(|dep| dep.get("version"))
        .map(|_| ())
        .ok_or_else(|| Error::bad_dependency(link, typ).into())
}

fn set_package_version(doc: &mut Document, version: &str) -> result::Result<(), CargoTomlError> {
    let table =
        get_top_table_mut(doc, "package").ok_or_else(|| CargoTomlError::no_table("package"))?;
    table_add_or_update_value(table, "version", version.into())
        .ok_or_else(|| CargoTomlError::no_value("version"))
}

fn set_dependency_version(table: &mut Table, version: &str, name: &str) -> Option<()> {
    match table.entry(name) {
        toml_edit::Entry::Occupied(mut req) => {
            let item = req.get_mut();

            if let Some(item) = item.as_inline_table_mut() {
                inline_table_add_or_update_value(item, "version", version.into());
                return Some(());
            }
            if let Some(item) = item.as_table_mut() {
                return table_add_or_update_value(item, "version", version.into());
            }

            None
        }
        toml_edit::Entry::Vacant(_) => Some(()),
    }
}

fn set_dependencies_version(
    doc: &mut Document,
    version: &str,
    typ: DependencyType,
    name: &str,
) -> result::Result<(), CargoTomlError> {
    if let Some(table) = get_top_table_mut(doc, typ.key()) {
        set_dependency_version(table, version, name)
            .ok_or_else(|| CargoTomlError::set_version(name, version))?;
    }

    if let Some(table) = get_top_table_mut(doc, "target") {
        let targets = table.iter().map(|(key, _)| key.to_owned()).collect_vec();

        for target in targets {
            let target_deps = table.entry(&target);
            match target_deps {
                toml_edit::Entry::Occupied(mut target_deps) => {
                    if let Some(target_deps) = target_deps
                        .get_mut()
                        .as_table_mut()
                        .and_then(|inner| inner[typ.key()].as_table_mut())
                    {
                        set_dependency_version(target_deps, version, name)
                            .ok_or_else(|| CargoTomlError::set_version(name, version))?;
                    }
                }
                toml_edit::Entry::Vacant(_) => {}
            };
        }
    };

    Ok(())
}

fn set_lockfile_self_describing_metadata(
    doc: &mut Document,
    next_release_version: &str,
    package_name: &str,
) -> result::Result<(), Error> {
    let packages_entry = doc.as_table_mut().entry("package");

    match packages_entry {
        toml_edit::Entry::Occupied(mut entry) => {
            let tables = entry
                .get_mut()
                .as_array_of_tables_mut()
                .expect("Expected lockfile to contain an array of tables named 'packages'");

            let matching_index = tables.iter().position(|table| {
                table
                    .get("name")
                    .and_then(|item| item.as_str())
                    .map(|name| name == package_name)
                    .unwrap_or_default()
            });

            if let Some(matching_index) = matching_index {
                let table = tables
                    .get_mut(matching_index)
                    .expect("Expected lockfile to contain reference to self");
                table_add_or_update_value(table, "version", next_release_version.into());
            } else {
                return Err(Error::CargoLockfileUpdate {
                    reason: "Unable to locate self-referential metadata in lockfile".into(),
                    package_name: package_name.to_owned(),
                });
            }
        }
        _ => {
            return Err(Error::CargoLockfileUpdate {
                reason: "Cargo lockfile does not contain 'packages' array of tables".into(),
                package_name: package_name.to_owned(),
            })
        }
    };

    Ok(())
}

/// The type of a dependency for a package.
#[derive(Debug)]
pub enum DependencyType {
    /// A normal dependency (i.e. "dependencies" section of `Cargo.toml`).
    Normal,

    /// A build dependency (i.e. "build-dependencies" section of `Cargo.toml`).
    Build,
}

impl DependencyType {
    fn key(&self) -> &str {
        use DependencyType::*;

        match self {
            Normal => "dependencies",
            Build => "build-dependencies",
        }
    }
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DependencyType::*;

        match self {
            Normal => write!(f, "Dependency"),
            Build => write!(f, "Build dependency"),
        }
    }
}

#[derive(Debug, Serialize)]
struct Release {
    name: String,
    url: Url,
}

impl Release {
    fn new(name: impl AsRef<str>, main_crate: impl AsRef<str>) -> Result<Self> {
        let base = Url::parse("https://crates.io/crates/").map_err(Error::url_parse_error)?;
        let url = base
            .join(main_crate.as_ref())
            .map_err(Error::url_parse_error)?;

        Ok(Self {
            name: name.as_ref().to_owned(),
            url,
        })
    }
}
