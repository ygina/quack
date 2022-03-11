extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/psum.c")
        .compile("libpsum.a");
    cc::Build::new()
        .file("src/ilp.c")
        .compile("libilp.a");
}
