// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Error};
use clap::{builder::TypedValueParser, crate_version, Parser};
use log::Level;

mod logger;

use semantic_release_cargo::{
    list_packages_with_arguments, prepare, publish, verify_conditions_with_alternate, PublishArgs,
};

/// Run semantic-release steps in the context of a cargo based Rust project.
#[derive(Parser)]
#[clap(version = crate_version!())]
struct Opt {
    /// Increases the logging level (use multiple times for more detail).
    #[clap(short, long, group = "logging", action = clap::ArgAction::Count)]
    verbose: u8,

    /// Explicitly set the log level.
    #[clap(
        short,
        long,
        group = "logging",
        value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace"])
            .map(|s| s.parse::<log::Level>().unwrap()),
    )]
    log_level: Option<Level>,

    /// Specifies the output file to use instead of standard out.
    #[structopt(short, long)]
    output: Option<PathBuf>,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    /// List the packages that are included in the semantic release.
    ///
    /// The listed packages are all of the packages in the workspace and are listed
    /// in order based on their dependencies (it is a topological sort of the
    /// dependency graph). Packages that will not be published will have such an
    /// indication given after the name of the package.
    ///
    /// This is primarily a debugging aid and does not corresponds directly to
    /// a semantic release step.
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
    /// This implements the `verifyConditions` step for `semantic-release` for a
    /// Cargo-based Rust workspace.
    #[clap(verbatim_doc_comment)]
    VerifyConditions(CommonOpt),

    /// Prepare the Rust workspace for a release.
    ///
    /// Preparing the workspace for a release updates the version of each crate in
    /// the workspace in the crate's `Cargo.toml` file, and adds or updates the
    /// version field of any workspace-relative path dependencies and
    /// build-dependencies.
    ///
    /// This implements the `prepare` step for `semantic-release` for a Cargo-based
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
    /// This implements the `publish` step for `semantic-release` for a Cargo-based
    /// Rust workspace.
    Publish(PublishOpt),
}

#[derive(Parser)]
struct CommonOpt {
    /// The path to the `Cargo.toml` file for the root of the workspace.
    #[clap(long)]
    manifest_path: Option<PathBuf>,

    /// Specify an alternate-registry to publish the target crate to.
    #[clap(long)]
    registry: Option<String>,
}

#[derive(Parser)]
struct PrepareOpt {
    #[clap(flatten)]
    common: CommonOpt,

    /// The version to set in all crates in the workspace.
    next_version: String,
}

#[derive(Parser)]
struct PublishOpt {
    #[clap(flatten)]
    common: CommonOpt,

    /// Disallow publishing with uncommited files in the workspace.
    #[clap(long)]
    no_dirty: bool,

    /// The features to use when publishing the workspace.
    /// This is a comma separated list of key-value pairs where the key is the
    /// name of the package and the value a feature for that package.
    /// For example, `--features foo=bar,baz=qux` will set the `bar` feature for
    /// the `foo` package and the `qux` feature for the `baz` package.
    #[clap(long, value_parser = parse_key_val::<String, String>, value_delimiter = ',')]
    features: Vec<(String, String)>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

impl Subcommand {
    fn run(&self, w: impl Write) -> Result<(), Error> {
        use Subcommand::*;

        match self {
            ListPackages(opt) => Ok(list_packages_with_arguments(
                w,
                opt.registry.as_deref(),
                opt.manifest_path(),
            )?),
            VerifyConditions(opt) => Ok(verify_conditions_with_alternate(
                w,
                opt.registry.as_deref(),
                opt.manifest_path(),
            )?),
            Prepare(opt) => Ok(prepare(
                w,
                opt.common.manifest_path(),
                opt.next_version.clone(),
            )?),
            Publish(opt) => Ok(publish(
                w,
                opt.common.manifest_path(),
                &PublishArgs {
                    no_dirty: Some(opt.no_dirty),
                    features: Some(opt.features.iter().cloned().fold(
                        Default::default(),
                        |mut a, (k, v)| {
                            a.entry(k).or_default().push(v);
                            a
                        },
                    )),
                    registry: opt.common.registry.clone(),
                },
            )?),
        }
    }
}

fn main() -> Result<(), Error> {
    let opt: Opt = Opt::parse();

    let log_builder = logger::LoggerBuilder::default()
        .output(Level::Trace, std::io::stderr())
        .output(Level::Debug, std::io::stderr());

    // Set the max level to initialize to based on the `log-level` flag if it's
    // available, otherwise fall back to verbosity.
    if let Some(log_level) = opt.log_level {
        log_builder.max_level(log_level).init()?;
    } else {
        log_builder.verbosity(opt.verbose).init()?;
    };

    match opt.output {
        Some(path) => {
            let file = File::create(&path)
                .with_context(|| format!("Failed to create output file {}", path.display()))?;
            opt.subcommand.run(BufWriter::new(file))
        }

        None => opt.subcommand.run(BufWriter::new(io::stdout())),
    }
}

impl CommonOpt {
    fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }
}
