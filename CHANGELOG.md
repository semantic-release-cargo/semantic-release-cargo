## [2.0.6](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.5...v2.0.6) (2022-12-21)


### Bug Fixes

* **deps:** update rust crate itertools to v0.10.5 ([6345ead](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/6345ead26e6a156c595315e29587da35d2eaceeb))
* **docs:** document supported systems ([0b71757](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/0b7175777600baa4573cbefbcfc4448fa75f0edd))

## [2.0.5](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.4...v2.0.5) (2022-12-21)


### Bug Fixes

* **deps:** update rust crate clap to v4.0.30 ([9c6e940](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/9c6e940a853dce75a28a61247373116b9fa8bc6d))

## [2.0.4](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.3...v2.0.4) (2022-12-21)


### Bug Fixes

* **deps:** update rust crate anyhow to v1.0.68 ([99aec13](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/99aec13a7a724e2ee33a46027bf18e59d0060fae))

## [2.0.3](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.2...v2.0.3) (2022-12-20)


### Bug Fixes

* correct regression in error handling ([f054630](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/f05463032bff6f5a24909e82cea3d7d0c7671c5b)), closes [#8](https://github.com/semantic-release-cargo/semantic-release-cargo/issues/8)

## [2.0.2](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.1...v2.0.2) (2022-12-20)

## [2.0.1](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v2.0.0...v2.0.1) (2022-12-20)

# [2.0.0](https://github.com/semantic-release-cargo/semantic-release-cargo/compare/v1.0.2...v2.0.0) (2022-12-19)


This is **not** a breaking change from semantic-release-rust@1.0.0-alpha.8, or from semantic-release-cargo@1.

This is only a feature update, I mistakenly bumped major versions on the beta branch and decided not to rewrite published history to avoid v2.0.0.

### Features

* feat: wrap rust code in npm package using napi-rs ([cedc828](https://github.com/semantic-release-cargo/semantic-release-cargo/commit/cedc828fbe8f2e889febbe02e248de11e7a459e9))
* support installation with cargo binstall

### Fixes

* use clap@v4 for argument parsing instead of structopt

# [2.0.0-beta.7](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.6...v2.0.0-beta.7) (2022-12-18)


### Bug Fixes

* avoid segfault in args parsing ([4c42566](https://github.com/semantic-release-rust/semantic-release-rust/commit/4c425665a8aada8d0a4053ad755f87db49d38100))
* remove CI artifacts from cargo package list ([764e04a](https://github.com/semantic-release-rust/semantic-release-rust/commit/764e04a0a73994ba4b06023886e98c96edeca43b))

# [2.0.0-beta.7](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.6...v2.0.0-beta.7) (2022-12-18)


### Bug Fixes

* avoid segfault in args parsing ([4c42566](https://github.com/semantic-release-rust/semantic-release-rust/commit/4c425665a8aada8d0a4053ad755f87db49d38100))

# [2.0.0-beta.6](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.5...v2.0.0-beta.6) (2022-12-18)


### Bug Fixes

* publish semantic-release-cargo to crates.io with semantic-release ([d89213b](https://github.com/semantic-release-rust/semantic-release-rust/commit/d89213b1adb9661764d2a29fa49b06bdceb9801f))

# [2.0.0-beta.5](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.4...v2.0.0-beta.5) (2022-12-18)


### Bug Fixes

* upload compiled CLI to GitHub Release artifacts ([34b692a](https://github.com/semantic-release-rust/semantic-release-rust/commit/34b692a32e1ffe9de53497f9300264a1fa974f21))

# [2.0.0-beta.4](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.3...v2.0.0-beta.4) (2022-12-18)


### Bug Fixes

* re-enable prepare lifecycle hook ([95dd885](https://github.com/semantic-release-rust/semantic-release-rust/commit/95dd88576bc8bf6b1a2c9c0bab7bf0f0ba100307))

# [2.0.0-beta.3](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.2...v2.0.0-beta.3) (2022-12-18)


### Bug Fixes

* use x86_64-unknown-linux-gnu target ([1517c0a](https://github.com/semantic-release-rust/semantic-release-rust/commit/1517c0ac4f7599f69616ac35b7fb340ba3f28bb0))

# [2.0.0-beta.2](https://github.com/semantic-release-rust/semantic-release-rust/compare/v2.0.0-beta.1...v2.0.0-beta.2) (2022-12-18)


### Bug Fixes

* remove prepare lifecycle hook ([f1bda08](https://github.com/semantic-release-rust/semantic-release-rust/commit/f1bda08cba102adf4d5536eaa9b59210441ae542))

# [2.0.0-beta.1](https://github.com/semantic-release-rust/semantic-release-rust/compare/v1.0.2...v2.0.0-beta.1) (2022-12-18)


* feat!: wrap rust code in npm package ([cedc828](https://github.com/semantic-release-rust/semantic-release-rust/commit/cedc828fbe8f2e889febbe02e248de11e7a459e9))


### BREAKING CHANGES

* Use napi-rs to call the Rust library from Node.js.

This deletes the Rust binary. Moving forward, **semantic-release-cargo**
should be installed by npm.
