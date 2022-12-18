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