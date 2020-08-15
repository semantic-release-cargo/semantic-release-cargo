// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::fs::File;
use std::io::{prelude::*, stdout, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use human_panic::setup_panic;
use log::Level;
use loggerv::{Logger, Output};
use structopt::StructOpt;

use semantic_release_rust::{list_packages, prepare, publish, verify_conditions};

/// Run sementic-release steps in the context of a cargo based Rust project.
#[derive(StructOpt)]
struct Opt {
    /// Increases the logging level (use multiple times for more detail).
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u64,

    /// Specifies the output file to use instead of standard out.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(StructOpt)]
enum Subcommand {
    /// List the packages that are included in the sementic release.
    ///
    /// The listed packages are all of the packages in the workspace and are listed
    /// in order based on their dependencies (it is a topological sort of the
    /// dependency graph). Packages that will not be published will have such an
    /// indication given after the name of the package.
    ///
    /// This is primarily a debuging aid and does not corresponde directly to a
    /// sementic release step.
    ListPackages(CommonOpt),

    /// Verify that the conditions for a release are satisfied
    ///
    /// The conditions for a release checked by this subcommand are:
    ///
    ///     1. That the CARGO_REGISTRY_TOKEN environment variable is set and is
    ///        non-empty.
    ///     2. That it can construct a reverse-dependencies-ordered list of the
    ///        packages in the root crate's workspace.
    ///     3. That it can parse the version for packages in the workspace in all of
    ///        the `Cargo.toml` files that form part of the workspace.
    ///
    /// This implments the `verifyConditions` step for `sementic-release` for a
    /// Cargo-based Rust workspace.
    #[structopt(verbatim_doc_comment)]
    VerifyConditions(CommonOpt),

    /// Prepare the Rust workspace for a release.
    ///
    /// Preparing the workspace for a release updates the version of each crate in
    /// the workspace in the crate's `Cargo.toml` file, and adds or updates the
    /// version field of any workspace-relative path dependencies and
    /// build-dependencies.
    ///
    /// This implments the `prepare` step for `semantic-release` for a Cargo-based
    /// Rust workspace.
    Prepare(PrepareOpt),

    /// Publish the Rust workspace.
    ///
    /// Publishing the workspace publishes each crate in the workspace to
    /// crates.io except crates with the `package.publish` field set to `false` or
    /// set to any registries other than just crates.io. By default this will publish
    /// with the `allow-dirty` flag but this can be excluded with the `no-dirty`
    /// flag to this subcommand.
    ///
    /// This implments the `publish` step for `semantic-release` for a Cargo-based
    /// Rust workspace.
    Publish(PublishOpt),
}

#[derive(StructOpt)]
struct CommonOpt {
    /// The path to the `Cargo.toml` file for the root of the workspace.
    #[structopt(long, parse(from_os_str))]
    manifest_path: Option<PathBuf>,
}

#[derive(StructOpt)]
struct PrepareOpt {
    #[structopt(flatten)]
    common: CommonOpt,

    /// The version to set in all crates in the workspace.
    next_version: String,
}

#[derive(StructOpt)]
struct PublishOpt {
    #[structopt(flatten)]
    common: CommonOpt,

    /// Disallow publishing with uncommited files in the workspace.
    #[structopt(long)]
    no_dirty: bool,
}

impl Subcommand {
    fn run(&self, w: impl Write) -> Result<(), Error> {
        use Subcommand::*;

        match self {
            ListPackages(opt) => Ok(list_packages(w, opt.manifest_path())?),
            VerifyConditions(opt) => Ok(verify_conditions(w, opt.manifest_path())?),
            Prepare(opt) => Ok(prepare(w, opt.common.manifest_path(), &opt.next_version)?),
            Publish(opt) => Ok(publish(w, opt.common.manifest_path(), opt.no_dirty)?),
        }
    }
}

fn main() -> Result<(), Error> {
    setup_panic!();

    let opt = Opt::from_args();

    Logger::new()
        .output(&Level::Trace, Output::Stderr)
        .output(&Level::Debug, Output::Stderr)
        .output(&Level::Info, Output::Stderr)
        .verbosity(opt.verbose)
        .init()?;

    match opt.output {
        Some(path) => {
            let file = File::create(&path)
                .with_context(|| format!("Failed to create output file {}", path.display()))?;
            opt.subcommand.run(BufWriter::new(file))
        }

        None => opt.subcommand.run(BufWriter::new(stdout())),
    }
}

impl CommonOpt {
    fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }
}
