fn main() {
    #[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
    cpp_build::Config::new().build("src/io/oiio.rs");

    #[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
    println!("cargo:rustc-link-lib=OpenImageIO");
}
