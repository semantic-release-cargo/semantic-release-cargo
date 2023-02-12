# semantic-release-cargo

[![Build Status]](https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml)

[build status]: https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml/badge.svg?event=push

**semantic-release-cargo** integrates a cargo-based Rust project with [semantic-release].
This solves two use cases:

1. publishing to [crates.io], and
2. compiling release binaries

[crates.io]: https://crates.io/
[semantic-release]: https://github.com/semantic-release/semantic-release

## Publish to crates.io

After following these instructions, you will have a semantic-release pipeline that publishes
your Rust crate to crates.io.

### Requirements

You must set the `CARGO_REGISTRY_TOKEN` environment variable.

This workflow is supported on the following systems:

| System        | Supported? |
| ------------- | ---------- |
| macOS aarch64 | ✓          |
| macOS x64     | ✓          |
| Linux x64 gnu | ✓          |

### Install

Install `semantic-release-cargo` with npm:

```bash
$ npm install --save-dev --save-exact @semantic-release-cargo/semantic-release-cargo
```

### Use

Add **semantic-release-cargo** to your `semantic-release` configuration in `.releaserc.json`:

```json
{
  "plugins": ["@semantic-release-cargo/semantic-release-cargo"]
}
```

### Alternative Configuration with semantic-release-action

If you're not keen to mix npm with your Rust project, you can use the [semantic-release-action].

[semantic-release-action]: https://github.com/cycjimmy/semantic-release-action

## Compile Release Binaries

After following these instructions, you will have a GitHub Actions workflow
that sets the next version number in `Cargo.toml` and compiles your crate's
release binaries.

Updating the cargo manifest with the next version number lets you reference
the next version in the compiled binary, for example with the [clap::crate_version]
macro.

The compiled binaries can be uploaded to a GitHub release using the
[@semantic-release/github] plugin.

This workflow is supported on the following systems:

| System        | Supported? |
| ------------- | ---------- |
| macOS aarch64 | ✓          |
| macOS x64     | ✓          |
| Linux aarch64 | ✓          |
| Linux x64     | ✓          |
| Linux x86     | ✓          |

[clap::crate_version]: https://docs.rs/clap/latest/clap/macro.crate_version.html
[@semantic-release/github]: https://github.com/semantic-release/github

### Use

In the first job, use the [next-release-version] Action to save the
next release version as GitHub Actions outputs:

```yaml
jobs:
  get-next-version:
    uses: semantic-release-action/next-release-version/.github/workflows/next-release-version.yml@v4
```

In the next job, use **semantic-release-cargo** to set the crate version before
compilation:

```yaml
build:
  name: Build
  runs-on: ubuntu-latest
  needs:
    - get-next-version

  steps:
    - name: Checkout
      uses: actions/checkout@v3

    - name: Install semantic-release-cargo
      if: needs.get-next-version.outputs.new-release-published == 'true'
      uses: EricCrosson/install-github-release-binary@v2
      with:
        targets: semantic-release-cargo@2

    - name: Prepare semantic-release for Rust
      if: needs.get-next-version.outputs.new-release-published == 'true'
      run: semantic-release-cargo prepare ${{ needs.get-next-version.outputs.new-release-version }}

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@master

    - name: Cargo build
      run: cargo build --release --target=x86_64-unknown-linux-gnu --all-targets
```

[next-release-version]: https://github.com/semantic-release-action/next-release-version

## Example Workflow

You can create a single GitHub Actions workflow that combines both use cases.
This repository uses **semantic-release-cargo** with semantic-release to publish
to crates.io and create a GitHub Release with precompiled binaries.

See [release.yml] for a working example.

[release.yml]: .github/workflows/release.yml

## Contributors License Agreement

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in **semantic-release-cargo** by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Acknowledgments

This repository has been forked from [kettleby/semantic-release-rust]. All
credit goes to the original author.

[kettleby/semantic-release-rust]: https://github.com/kettleby/semantic-release-rust
