fn main() {
    cc::Build::new()
        .flag("-Wno-everything")
        .file("./bcdec/test.c")
        .include("./bcdec/")
        .compile("bcdec");
}
