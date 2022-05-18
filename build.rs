fn main() {
    #[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
    {
        cpp_build::Config::new()
            .flag("-std=c++14")
            .build("src/io/oiio.rs");

        println!("cargo:rustc-link-lib=OpenImageIO");
        println!("cargo:rustc-link-lib=OpenImageIO_Util");
    }
}
