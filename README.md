# Semantic Release--Rust

**semantic-release-rust integrates a cargo based project into semantic-release**
[![Build Status](https://img.shields.io/github/workflow/status/sbosnick/semantic-release-rust/CI)](https://github.com/sbosnick/semantic-release-rust/actions?query=workflow%3ACI)
[![Latest Version](https://img.shields.io/crates/v/semantic-release-rust)](https://crates.io/crates/semantic-release-rust)
[![Documentation](https://img.shields.io/badge/api-rustdoc-blue)](https://doc.rs/semantic-release-rust)
[![semantic-release](https://img.shields.io/badge/%20%20%F0%9F%93%A6%F0%9F%9A%80-semantic--release-e10079.svg)](https://github.com/semantic-release/semantic-release)
---

Semantic Release Rust integrates a cargo-based Rust project into [semantic-release].
Specifically it provides submcommand for each of the `verifyConditons`, `prepare`,
and `publish` step of [semantic-release].

[semantic-release]: https://github.com/semantic-release/semantic-release

## Usage
Install `semantic-release-rust` with

```bash
$ cargo install semantic-release-rust
```

then add it to your `semantic-release` configuration using the [`semantic-release/exec`][exec]
plugin. For example in `.releaserc.yml`:

```yaml
plugins:
    - '@semantic-release/commit-analyzer'
    - '@semantic-release/release-notes-generator'
    - '@semantic-release/github'
    - - '@semantic-release/exec'
      - verifyConditionsCmd: "./target/release/semantic-release-rust verify-conditions"
        prepareCmd: "./target/release/semantic-release-rust prepare"
        publishCmd: "./target/release/semantic-release-rust publish"
```

`semantic-release-rust` expects (and verifies) that the environment variable
`CARGO_REGISTRY_TOKEN` is set. It should be set to an API Access token for `crates.io`
access. You likely want to set this through the secrets mechanims of your CI provider.

[exec]: https://github.com/semantic-release/exec

## License

Semantic Release Rust is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE-2.0](LICENSE-APACHE-2.0) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

## Contribution

Please note that this project is released with a [Contributor Code of
Conduct][code-of-conduct].  By participating in this project you agree to abide
by its terms.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Semantic Release Rust by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[code-of-conduct]: CODE_OF_CONDUCT.md
