fn main() {
    println!("cargo:rustc-link-lib=dylib=omp");
    println!("cargo:rustc-link-lib=dylib=faiss");
    cpp_build::Config::new().build("src/lib.rs");
}
