fn main() {
    cc::Build::new()
        .file("stb/stb.c")
        .flag("-Wno-unused-parameter")
        .compile("stb")
}
