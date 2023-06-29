fn main() {
    // link the EGL library
    println!("cargo:rustc-link-lib=dylib=EGL");
}
