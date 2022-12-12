# Semantic Release Cargo

[![Build Status]](https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml)

[build status]: https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml/badge.svg?event=push

**semantic-release-cargo** integrates a cargo-based Rust project into [semantic-release].
Specifically it provides sub-command for each of the `verifyConditons`, `prepare`,
and `publish` steps of [semantic-release].

[semantic-release]: https://github.com/semantic-release/semantic-release

## Usage

Install `semantic-release-cargo` with

```bash
$ cargo install semantic-release-cargo
```

then add it to your `semantic-release` configuration using the [`semantic-release/exec`][exec]
plugin. For example, in `.releaserc.json`:

```json
{
  "plugins": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/github",
    [
      "@semantic-release/exec",
      {
        "verifyConditionsCmd": "semantic-release-cargo verify-conditions",
        "prepareCmd": "semantic-release-cargo prepare ${nextRelease.version}",
        "publishCmd": "semantic-release-cargo publish"
      }
    ]
  ]
}
```

`semantic-release-cargo` expects (and verifies) that the environment variable
`CARGO_REGISTRY_TOKEN` is set. It should be set to an API Access token for `crates.io`
access. You likely want to set this through the secrets mechanism of your CI provider.

[exec]: https://github.com/semantic-release/exec

## License

**semantic-release-cargo** is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

## Contribution

Please note that this project is released with a [Contributor Code of
Conduct][code-of-conduct]. By participating in this project you agree to abide
by its terms.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in **semantic-release-cargo** by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[code-of-conduct]: CODE_OF_CONDUCT.md
