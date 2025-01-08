mod generator;
mod parser;

use std::path::{Path, PathBuf};

pub fn build(sv_path: &Path) -> Result<PathBuf, String> {
    let probe = parser::parse(sv_path)?;
    build_with_probe(sv_path, &probe)
}

pub fn build_with_probe(sv_path: &Path, probe: &parser::Probe) -> Result<PathBuf, String> {
    let build_path = generator::generate(sv_path, probe)?;
    cmake_build(&build_path)
}

fn cmake_build(_build_path: &Path) -> Result<PathBuf, String> {
    // temporarility return sample dut
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.push("sample/build/libdut.so");
    Ok(lib_path)
}
