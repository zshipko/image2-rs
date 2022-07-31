fn main() {
    #[cfg(all(feature = "oiio", not(feature = "docs-rs")))]
    {
        let mut pkg = std::process::Command::new("pkg-config");
        pkg.arg("--cflags")
            .arg("--libs")
            .arg("--silence-errors")
            .arg("OpenImageIO");

        if cfg!(target_os = "macos") {
            pkg.arg("Imath");
        }

        let pkg = pkg.output().unwrap();
        let flags = String::from_utf8(pkg.stdout).unwrap();
        let flags = flags.split(" ");
        let mut config = cpp_build::Config::new();
        config
            .flag("-std=c++14")
            .include("/opt/homebrew/Cellar/openimageio/2.3.17.0/include");

        for flag in flags {
            config.flag(flag);
        }

        config.build("src/io/oiio.rs");

        println!("cargo:rustc-link-lib=OpenImageIO");
        println!("cargo:rustc-link-lib=OpenImageIO_Util");
    }
}
