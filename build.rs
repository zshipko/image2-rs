fn main() {
    cpp_build::Config::new().build("src/oiio.rs");
    println!("cargo:rustc-link-lib=OpenImageIO");
}
