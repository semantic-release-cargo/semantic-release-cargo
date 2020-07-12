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

use std::env;
use std::io::Write;
use std::path::PathBuf;

use guppy::{MetadataCommand, graph::{DependencyDirection, PackageGraph}};
use log::{debug, info, error};

mod error;

pub use error::{Error, Result};

/// List the packages from the workspace in the order of their dependencies.
///
/// The list of pacakges will be written to `output`. If `manifest_path` is provided
/// then it is expected to give the path to the `Cargo.toml` file for the root of the
/// workspace. If `manifest_path` is `None` then `list_packages` will look for the
/// root of the workspace in a `Cargo.toml` file in the current directory.
///
/// This is a debuging aid and does not directly correspond to a a sementic release
/// step.
pub fn list_packages(mut output: impl Write, manifest_path: Option<&PathBuf>) -> Result<()> {
    let graph = get_package_graph(manifest_path)?;

    info!("Resolving workspace");
    let set = graph.resolve_workspace();

    info!("Listing packages");
    for package in set.packages(DependencyDirection::Reverse) {
        write!(output, "{}({})", package.name(), package.version()).map_err(Error::output_error)?;
        if !package.publish().is_none() {
            write!(output, "--not published").map_err(Error::output_error)?;
        }
        writeln!(output).map_err(Error::output_error)?;
    }

    Ok(())
}

fn get_package_graph(manifest_path: Option<&PathBuf>) -> Result<PackageGraph> {
    let mut command = MetadataCommand::new();
    if let Some(path) = manifest_path {
        command.manifest_path(path);
    }

    info!("Building package graph");
    debug!("manifest_path: {:?}", manifest_path);

    command.build_graph().map_err(|err| {
        let path = match manifest_path {
            Some(path) => path.clone(),
            None => env::current_dir()
                .map(|path| path.join("Cargo.toml"))
                .unwrap_or_else(|e| {
                    error!("Unable to get current directory: {}", e);
                    PathBuf::from("unknown manifest")
                }),
        };
        Error::workspace_error(err, path)
    })
}
