fn main() {
    #[cfg(feature = "oiio")]
    {
        cpp_build::Config::new().build("src/io/oiio.rs");

        if cfg!(not(feature = "docs-rs")) {
            println!("cargo:rustc-link-lib=OpenImageIO");
        }
    }
}
