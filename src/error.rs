// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::{Path, PathBuf};

use guppy::errors::Error as GuppyError;
use guppy::graph::PackageLink;
use thiserror::Error;
use toml_edit::TomlError as TomlEditError;

use super::DependencyType;

/// The error type for operations `sementic-release-rust` operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while parsing the structure of a workspace.
    #[error(transparent)]
    WorkspaceError(WorkspaceError),

    /// Error while writing to the output.
    #[error("Unable to write to the output")]
    OutputError(#[source] io::Error),

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
    BadDependancy {
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

        /// The `Cargo.toml` file in which the error occured.
        manifest_path: PathBuf,
    },
}

/// A specialized `Result` type for `semantic-release-rust` operations.
pub type Result<T> = std::result::Result<T, Error>;

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

impl Error {
    pub(crate) fn workspace_error(metadata_error: GuppyError, manifest_path: PathBuf) -> Error {
        Error::WorkspaceError(WorkspaceError {
            metadata_error,
            manifest_path,
        })
    }

    pub(crate) fn output_error(inner: io::Error) -> Error {
        Error::OutputError(inner)
    }

    pub(crate) fn verify_error(reason: impl Into<String>) -> Error {
        Error::VerifyError {
            reason: reason.into(),
        }
    }

    pub(crate) fn bad_dependency(link: &PackageLink, typ: DependencyType) -> Error {
        Error::BadDependancy {
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
