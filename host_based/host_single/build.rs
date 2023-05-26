fn main() {
    cc::Build::new()
        .file("src/valgrind.c")
        .compile("valgrind");
    println!("cargo:rerun-if-changed=src/valgrind.c");
    println!("cargo:rustc-link-lib=valgrind");
}