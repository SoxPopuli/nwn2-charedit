fn main() {
    cc::Build::new()
        .warnings(false)
        .file("bcdec.c")
        .include("./bcdec/")
        .compile("bcdec");
}
