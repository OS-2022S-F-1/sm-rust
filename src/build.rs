extern crate cc;

fn main() {
    cc::Build::new()
        .file("mprv.S")
        .compile("mprv-asm-lib");
}