mod error;
mod generator;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use error::OombakGenResult;
use oombak_rs::probe::Probe;

pub fn build(sv_path: &Path) -> OombakGenResult<PathBuf> {
    let source_paths = [
        "/home/fuad1502/code/oombak_parser/tests/fixtures/sv_sample_1/sample.sv",
        "/home/fuad1502/code/oombak_parser/tests/fixtures/sv_sample_1/adder.sv",
    ];
    let probe = Probe::try_from(&source_paths, "sample")?;
    build_with_probe(sv_path, &probe)
}

pub fn build_with_probe(sv_path: &Path, probe: &Probe) -> OombakGenResult<PathBuf> {
    let source_path = generator::generate(sv_path, probe)?;
    Ok(cmake(&source_path)?)
}

fn cmake(source_path: &Path) -> OombakGenResult<PathBuf> {
    cmake_configure(source_path)?;
    cmake_build(source_path)?;
    let mut so_path = PathBuf::from(source_path);
    so_path.push("build");
    so_path.push("libdut.so");
    Ok(so_path)
}

fn cmake_configure(source_path: &Path) -> OombakGenResult<()> {
    Command::new("cmake")
        .current_dir(source_path)
        .args(["-S", ".", "-B", "build"])
        .output()?;
    Ok(())
}

fn cmake_build(source_path: &Path) -> OombakGenResult<()> {
    Command::new("cmake")
        .current_dir(source_path)
        .args(["--build", "build"])
        .output()?;
    Ok(())
}
