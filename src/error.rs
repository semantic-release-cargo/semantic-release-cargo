// Copyright 2020 Steven Bosnick
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE-2.0 or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::PathBuf;

use guppy::errors::Error as GuppyError;
use thiserror::Error;

/// The error type for operations `sementic-release-rust` operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while parsing the structure of a workspace.
    #[error(transparent)]
    WorkspaceError(WorkspaceError),

    /// Error while writing to the output.
    #[error("Unable to write to the output")]
    OutputError(#[source] io::Error),
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
}
