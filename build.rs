fn main() {
    cc::Build::new()
        .file("stb/stb.c")
        .flag_if_supported("-Wno-unused-parameter")
        .compile("stb");
}
