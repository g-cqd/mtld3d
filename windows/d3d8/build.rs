fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
    println!("cargo:rustc-cdylib-link-arg=/DEF:{manifest_dir}/d3d8.def");
    println!("cargo:rerun-if-changed=d3d8.def");
}
