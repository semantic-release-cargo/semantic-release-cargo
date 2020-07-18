// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::PathBuf;

use guppy::graph::PackageLink;
use guppy::errors::Error as GuppyError;
use thiserror::Error;

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
    VerifyError{
        /// The reason the conditions are not satisfied.
        reason: String,
    },

    /// Error while verifying that dependencies allow publication.
    ///
    /// This is a specific part of verifying the conditions for a release.
    #[error("{typ} of {from} on {to} prevents publication of {from}")]
    BadDependancy{
        /// The name of the package whose dependency prevents publication.
        from: String,

        /// The depended on package that prevents publication.
        to: String,

        /// The type of dependency that prevents publication.
        typ: DependencyType,
    }
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
}

