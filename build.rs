fn main() {
    if get_os_type() == "macos" {
        println!("cargo:rustc-link-lib=dylib=omp");
        println!("cargo:rustc-link-lib=dylib=faiss");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=faiss");
        println!("cargo:rustc-link-lib=dylib=gomp");
        println!("cargo:rustc-link-lib=dylib=blas");
        println!("cargo:rustc-link-lib=dylib=lapack");
    }
    cpp_build::Config::new().build("src/lib.rs");
}

fn get_os_type() -> &'static str {
    if cfg!(target_os = "windows") {
        return "windows";
    } else if cfg!(target_os = "linux") {
        return "linux";
    } else if cfg!(target_os = "macos") {
        return "macos";
    } else {
        return "unknown_os";
    }
}
