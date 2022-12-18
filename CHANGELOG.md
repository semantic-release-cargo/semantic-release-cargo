# [2.0.0-beta.1](https://github.com/semantic-release-rust/semantic-release-rust/compare/v1.0.2...v2.0.0-beta.1) (2022-12-18)


* feat!: wrap rust code in npm package ([cedc828](https://github.com/semantic-release-rust/semantic-release-rust/commit/cedc828fbe8f2e889febbe02e248de11e7a459e9))


### BREAKING CHANGES

* Use napi-rs to call the Rust library from Node.js.

This deletes the Rust binary. Moving forward, **semantic-release-cargo**
should be installed by npm.
