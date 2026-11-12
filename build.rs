//! Empty: Cargo requires a build script for a package that sets `links`,
//! and `links = "define_err_facade"` in `Cargo.toml` is what makes a second
//! copy of this crate in one build a resolve error. It builds nothing, reads
//! nothing, and runs once.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
