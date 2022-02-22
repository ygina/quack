extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/psum.c")
        .compile("libpsum.a");
}
