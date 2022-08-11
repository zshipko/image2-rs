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
        let flags = flags.split(' ');
        let mut config = cpp_build::Config::new();
        config.flag("-std=c++14").flag("-w");

        let mut search = Vec::new();
        let mut libs = Vec::new();

        for flag in flags {
            if let Some(s) = flag.strip_prefix("-L") {
                search.push(s.to_string());
            } else if let Some(s) = flag.strip_prefix("-l") {
                libs.push(s.to_string());
            } else {
                config.flag(flag);
            }
        }

        config.build("src/io/oiio.rs");

        for s in search {
            println!("cargo:rustc-link-search={s}");
        }

        if libs.is_empty() {
            println!("cargo:rustc-link-lib=OpenImageIO");
            println!("cargo:rustc-link-lib=OpenImageIO_Util");
        } else {
            for lib in libs {
                println!("cargo:rustc-link-lib={lib}");
            }
        }
    }
}
