# semantic-release-cargo

[![Build Status]](https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml)

[build status]: https://github.com/semantic-release-cargo/semantic-release-cargo/actions/workflows/release.yml/badge.svg?event=push

**semantic-release-cargo** integrates a cargo-based Rust project with [semantic-release].
Specifically it provides sub-command for each of the `verifyConditons`, `prepare`,
and `publish` steps of [semantic-release].

[semantic-release]: https://github.com/semantic-release/semantic-release

## Install

Install `semantic-release-cargo` with npm:

```bash
$ npm install --save-dev --save-exact @semantic-release-cargo/semantic-release-cargo
```

## Use

Add **semantic-release-cargo** to your `semantic-release` configuration using the [`semantic-release/exec`][exec]
plugin. For example, in `.releaserc.json`:

```json
{
  "plugins": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/github",
    "@semantic-release-cargo/semantic-release-cargo"
  ]
}
```

`semantic-release-cargo` expects (and verifies) that the environment variable
`CARGO_REGISTRY_TOKEN` is set. It should be set to an API Access token for `crates.io`
access. You likely want to set this through the secrets mechanism of your CI provider.

[exec]: https://github.com/semantic-release/exec

### Use with semantic-release-action

If you're not keen to mix npm with your Rust project, you can use the [semantic-release-action].

[Here][semantic-release-action-example] is an example using semantic-release-action, presented with
the disclaimer that I'm not familiar with this action myself.

[semantic-release-action]: https://github.com/cycjimmy/semantic-release-action
[semantic-release-action-example]: https://github.com/kettleby/semantic-release-rust/blob/2b183b27fac6abe54ca7741498e5f7a222ad07bb/.github/workflows/release.yml#L38-L45

## Contributors License Agreement

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in **semantic-release-cargo** by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

## Acknowledgments

This repository has been forked from [kettleby/semantic-release-rust]. All
credit goes to the original author.

[kettleby/semantic-release-rust]: https://github.com/kettleby/semantic-release-rust
