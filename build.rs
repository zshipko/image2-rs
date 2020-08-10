fn main() {
    cpp_build::Config::new().build("src/io.rs");

    if cfg!(not(feature = "docs-rs")) {
        println!("cargo:rustc-link-lib=OpenImageIO");
    }
}
