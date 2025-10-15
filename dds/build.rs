fn main() {
    cc::Build::new()
        .cpp(true)
        .std("c++11")
        .warnings(false)
        .files(["./bc7enc_rdo/bc7decomp.cpp", "bridge.cpp"])
        .include("./bc7enc_rdo/")
        .compile("bc7enc_rdo");
}
