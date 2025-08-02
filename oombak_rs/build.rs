fn main() {
    let dst = cmake::Config::new("oombak_parser").build();

    println!("cargo::rerun-if-changed=oombak_parser");
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!(
        "cargo:rustc-link-search=native={}/build/_deps/slang-build/lib",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=oombak_parser");
    println!("cargo:rustc-link-lib=static=svlang");
    println!("cargo:rustc-link-lib=static=fmtd");
    println!("cargo:rustc-link-lib=static=mimalloc-debug");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}
