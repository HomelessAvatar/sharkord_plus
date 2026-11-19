fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("android") {
        // Link LLVM libc++ shared library to resolve C++ exception unwinding (__gxx_personality_v0)
        println!("cargo:rustc-link-lib=c++_shared");
        // Enforce 16 KB page-size alignment for ELF PT_LOAD segments required by Android 15 & Google Pixel 8/9 devices
        println!("cargo:rustc-link-arg=-Wl,-z,max-page-size=16384");
        println!("cargo:rustc-link-arg=-Wl,-z,common-page-size=16384");
    }
    tauri_build::build();
}
