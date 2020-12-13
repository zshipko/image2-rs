fn main() {
    #[cfg(feature = "oiio")]
    cpp_build::Config::new().build("src/io/oiio.rs");

    #[cfg(feature = "oiio")]
    println!("cargo:rustc-link-lib=OpenImageIO");
}
