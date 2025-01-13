mod error;
mod generator;
mod parser;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn build(sv_path: &Path) -> Result<PathBuf, String> {
    let probe = parser::parse(sv_path)?;
    build_with_probe(sv_path, &probe)
}

pub fn build_with_probe(sv_path: &Path, probe: &parser::Probe) -> Result<PathBuf, String> {
    let source_path = generator::generate(sv_path, probe)?;
    Ok(cmake(&source_path)?)
}

fn cmake(source_path: &Path) -> error::OombakGenResult<PathBuf> {
    cmake_configure(source_path)?;
    cmake_build(source_path)?;
    let mut so_path = PathBuf::from(source_path);
    so_path.push("build");
    so_path.push("libdut.so");
    Ok(so_path)
}

fn cmake_configure(source_path: &Path) -> error::OombakGenResult<()> {
    Command::new("cmake")
        .current_dir(source_path)
        .args(["-S", ".", "-B", "build"])
        .output()?;
    Ok(())
}

fn cmake_build(source_path: &Path) -> error::OombakGenResult<()> {
    Command::new("cmake")
        .current_dir(source_path)
        .args(["--build", "build"])
        .output()?;
    Ok(())
}
