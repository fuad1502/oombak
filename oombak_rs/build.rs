use rerun_except::rerun_except;

fn main() {
    rerun_except(&["build/", "compile_commands.json", ".cache/", "target/"]).unwrap();

    let dst = cmake::Config::new("oombak_parser").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!(
        "cargo:rustc-link-search=native={}/build/_deps/slang-build/lib",
        dst.display()
    );

    println!("cargo:rustc-link-lib=static=oombak_parser");
    println!("cargo:rustc-link-lib=static=svlang");
    println!("cargo:rustc-link-lib=static=fmtd");
    println!("cargo:rustc-link-lib=static=mimalloc-debug");
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");
}
