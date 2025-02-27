// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    io,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use guppy::errors::Error as GuppyError;
use guppy::graph::PackageLink;
use thiserror::Error;
use toml_edit::TomlError as TomlEditError;
use url::ParseError;

use super::DependencyType;

/// The error type for operations `semantic-release-rust` operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while parsing the structure of a workspace.
    #[error(transparent)]
    WorkspaceError(WorkspaceError),

    /// Error when verifying that a workspace does not include cycles.
    #[error("Workspace has at least one cycle that includes as least {crate1} and {crate2}")]
    WorkspaceCycles {
        /// The first crate in the cycle.
        crate1: String,

        /// The second crate in the cycle.
        crate2: String,
    },

    /// Error while verifying the conditions for a release.
    #[error("Conditions for a release are not satisfied: {reason}")]
    VerifyError {
        /// The reason the conditions are not satisfied.
        reason: String,
    },

    /// Error while verifying that dependencies allow publication.
    ///
    /// This is a specific part of verifying the conditions for a release.
    #[error("{typ} of {from} on {to} prevents publication of {from}")]
    BadDependency {
        /// The name of the package whose dependency prevents publication.
        from: String,

        /// The depended on package that prevents publication.
        to: String,

        /// The type of dependency that prevents publication.
        typ: DependencyType,
    },

    /// Error while reading a file.
    #[error("Unable to read file {}", path.display())]
    FileReadError {
        /// The underlying error.
        #[source]
        inner: io::Error,

        /// The path that could not be read.
        path: PathBuf,
    },

    /// Error while writing a file.
    #[error("Unable to write file {}", path.display())]
    FileWriteError {
        /// The underlying error.
        #[source]
        inner: io::Error,

        /// The path the could not be written.
        path: PathBuf,
    },

    /// Error while parsing a TOML document.
    #[error(transparent)]
    TomlError(TomlError),

    /// Error while examining the contents of a `Cargo.toml` file.
    #[error("Unexpected contents of {manifest_path}")]
    CargoTomlError {
        /// The error found in the `Cargo.toml` file.
        #[source]
        inner: CargoTomlError,

        /// The `Cargo.toml` file in which the error occurred.
        manifest_path: PathBuf,
    },

    /// Error while attempting to run `cargo publish`
    #[error("Unable to run \"cargo publish\" for {manifest_path}")]
    CargoPublish {
        /// The underlying error.
        #[source]
        inner: io::Error,

        /// The manifest path for the crate on which the error occurred.
        manifest_path: PathBuf,
    },

    /// Error that records a non-sucess exit status from `cargo publish`.
    #[error("\"cargo publish\" exited with a failure for {manifest_path}: {status}\n{stderr}")]
    CargoPublishStatus {
        /// The exit status from `cargo publish`.
        status: ExitStatus,

        /// The manifest path for the crate on which the error occurred.
        manifest_path: PathBuf,

        /// The stderr output from cargo publish
        stderr: String,
    },

    /// Error while parsing a url for the release record.
    #[error(transparent)]
    UrlError(UrlError),

    /// Error while attempting to write the release record as JSON.
    #[error(transparent)]
    WriteReleaseError(WriteReleaseError),

    /// Error while attempting to update Cargo lockfile.
    #[error("Unable to update Cargo lockfile")]
    CargoLockfileUpdate {
        /// The reason for the failed lockfile update.
        reason: String,
        /// The package name where lockfile updating failed.
        package_name: String,
    },
}

/// A specialized `Result` type for `semantic-release-cargo` operations.
// #[cfg(feature = "napi-rs")]
pub type Result<T> = std::result::Result<T, anyhow::Error>;
// #[cfg(not(feature = "napi-rs"))]
// pub type Result<T> = std::result::Result<T, Error>;

/// The error details related to a problem parsing the workspace structure.
#[derive(Debug, Error)]
#[error("Unable to parse the workspace structure starting at {manifest_path}")]
pub struct WorkspaceError {
    #[source]
    metadata_error: GuppyError,
    manifest_path: PathBuf,
}

/// The error details related to a problem parsing a TOML file.
#[derive(Debug, Error)]
#[error("Unable to parse {} as a TOML file", path.display())]
pub struct TomlError {
    #[source]
    inner: TomlEditError,
    path: PathBuf,
}

/// The error details related the contents of a `Cargo.toml` file.
#[derive(Debug, Error)]
pub enum CargoTomlError {
    /// Error related to a missing table in a `Cargo.toml` file.
    #[error("Unable to locate expected table {table_name}")]
    NoTable {
        /// The name of the missing table.
        table_name: String,
    },

    /// Error related to a missing value in a `Cargo.toml` file.
    #[error("Unable to located expected value {value_name}")]
    NoValue {
        /// The name of the missing value.
        value_name: String,
    },

    /// Error related to failed attempt to set the version for a package or a dependency.
    #[error("Unable to set the version for {name} to {version}")]
    SetVersion {
        /// The name of the package or dependency.
        name: String,

        /// The version to which we attempted to set for the package or dependency.
        version: String,
    },
}

/// The error details related to an error parsing a url.
#[derive(Debug, Error)]
#[error("Unable to parse url for displaying release record.")]
pub struct UrlError {
    #[source]
    inner: ParseError,
}

/// The error details related to writing a release record as JSON.
#[derive(Debug, Error)]
#[error("Unable to write the release record for {main_crate} as JSON.")]
pub struct WriteReleaseError {
    #[source]
    inner: serde_json::Error,

    main_crate: String,
}

impl Error {
    pub(crate) fn workspace_error(metadata_error: GuppyError, manifest_path: PathBuf) -> Error {
        Error::WorkspaceError(WorkspaceError {
            metadata_error,
            manifest_path,
        })
    }

    pub(crate) fn verify_error(reason: impl Into<String>) -> Error {
        Error::VerifyError {
            reason: reason.into(),
        }
    }

    pub(crate) fn bad_dependency(link: &PackageLink, typ: DependencyType) -> Error {
        Error::BadDependency {
            from: link.from().name().to_string(),
            to: link.to().name().to_string(),
            typ,
        }
    }

    pub(crate) fn file_read_error(inner: io::Error, path: impl AsRef<Path>) -> Error {
        Error::FileReadError {
            inner,
            path: path.as_ref().to_owned(),
        }
    }

    pub(crate) fn file_write_error(inner: io::Error, path: impl AsRef<Path>) -> Error {
        Error::FileWriteError {
            inner,
            path: path.as_ref().to_owned(),
        }
    }

    pub(crate) fn toml_error(inner: TomlEditError, path: impl AsRef<Path>) -> Error {
        Error::TomlError(TomlError {
            inner,
            path: path.as_ref().to_owned(),
        })
    }

    pub(crate) fn cargo_publish(inner: io::Error, manifest_path: &Path) -> Error {
        Error::CargoPublish {
            inner,
            manifest_path: manifest_path.to_owned(),
        }
    }

    pub(crate) fn cargo_publish_status(
        status: ExitStatus,
        manifest_path: &Path,
        stderr: &[u8],
    ) -> Error {
        Error::CargoPublishStatus {
            status,
            manifest_path: manifest_path.to_owned(),
            stderr: String::from_utf8_lossy(stderr).into_owned(),
        }
    }

    pub(crate) fn url_parse_error(inner: ParseError) -> Error {
        Error::UrlError(UrlError { inner })
    }

    pub(crate) fn write_release_error(inner: serde_json::Error, main_crate: &str) -> Error {
        Error::WriteReleaseError(WriteReleaseError {
            inner,
            main_crate: main_crate.to_owned(),
        })
    }
}

impl CargoTomlError {
    pub(crate) fn no_table(table: &str) -> Self {
        Self::NoTable {
            table_name: table.to_owned(),
        }
    }

    pub(crate) fn no_value(value: &str) -> Self {
        Self::NoValue {
            value_name: value.to_owned(),
        }
    }

    pub(crate) fn set_version(name: &str, version: &str) -> Self {
        Self::SetVersion {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    pub(crate) fn into_error(self, path: impl AsRef<Path>) -> Error {
        Error::CargoTomlError {
            inner: self,
            manifest_path: path.as_ref().to_owned(),
        }
    }
}
