#[cfg(feature = "libpari")]
extern crate cc;

fn main() {
    #[cfg(feature = "libpari")]
    {
        cc::Build::new()
            .file("src/arithmetic/factor.c")
            .compile("libfactor.a");
        println!("cargo:rustc-link-lib=dylib=pari");
    }
}
