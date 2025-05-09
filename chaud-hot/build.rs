#![allow(
    clippy::expect_used,
    reason = "less restrictions on internal build-time dependencies"
)]

use std::env;

fn main() {
    let mut out_dir = env::var("OUT_DIR").expect("OUT_DIR to be valid UTF8");
    // `chaud-rustc` uses this marker to detect when hot reloading is enabled.
    out_dir.push_str("/__chaud_hot_marker");

    println!("cargo::rustc-link-search=crate={out_dir}");
}
