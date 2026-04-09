//! Emits a compiler warning when this crate is built as a dependency.

fn main() {
    println!("cargo:warning=crepuscularity: UNSTABLE / in active development — APIs may change without a semver-major bump until 1.0. Pin exact versions.");
}
