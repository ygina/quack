#[cfg(not(feature = "disable_validation"))]
extern crate cc;

fn main() {
    #[cfg(not(feature = "disable_validation"))]
    {
        cc::Build::new()
            .file("src/psum.c")
            .compile("libpsum.a");
        cc::Build::new()
            .file("src/ilp.c")
            .compile("libilp.a");
    }
}
