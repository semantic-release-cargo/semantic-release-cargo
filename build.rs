#[cfg(feature = "napi-rs")]
extern crate napi_build;

fn main() {
    #[cfg(feature = "napi-rs")]
    napi_build::setup();
}
