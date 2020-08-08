fn main() {
    cpp_build::Config::new().build("src/io.rs");
    println!("cargo:rustc-link-lib=OpenImageIO");
}
