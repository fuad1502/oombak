pub mod error;
mod generator;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use error::{OombakGenError, OombakGenResult};
use oombak_rs::probe::Probe;

pub fn build(sv_path: &Path) -> OombakGenResult<(PathBuf, Probe)> {
    let source_paths: Vec<String> = source_paths_from_sv_path(sv_path)?
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let probe = Probe::try_from(&source_paths, "sample")?;
    Ok((build_with_probe(sv_path, &probe)?, probe))
}

pub fn build_with_probe(sv_path: &Path, probe: &Probe) -> OombakGenResult<PathBuf> {
    let source_path = generator::generate(sv_path, probe)?;
    cmake(&source_path)
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

fn source_paths_from_sv_path(sv_path: &Path) -> OombakGenResult<Vec<PathBuf>> {
    if !sv_path.exists() || !sv_path.is_file() {
        return Err(OombakGenError::SvFilePathNotFound(sv_path.to_path_buf()));
    }
    let mut source_paths = vec![];
    source_paths.push(sv_path.to_path_buf());
    let parent_dir = sv_path
        .parent()
        .ok_or(OombakGenError::InvalidPath(sv_path.to_path_buf()))?;
    for file in std::fs::read_dir(parent_dir)? {
        let file = file?;
        if let Some(ext) = file.path().extension() {
            if ext == "sv" && file.path() != sv_path {
                source_paths.push(file.path())
            }
        }
    }
    Ok(source_paths)
}
