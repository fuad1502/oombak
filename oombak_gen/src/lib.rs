pub mod error;
mod generator;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use oombak_rs::probe::Probe;
use oombak_sim::{response::Percentage, Message};
use tempfile::TempDir;
use tokio::sync::mpsc::Sender;

pub use error::{Error, OombakGenResult};

pub struct TempGenDir {
    tempdir: TempDir,
    lib_path: PathBuf,
}

pub struct Builder {
    notification_channel: Option<Sender<Message>>,
    message_id: usize,
    progress: Percentage,
}

impl Builder {
    pub fn new(notification_channel: Option<Sender<Message>>, message_id: usize) -> Self {
        Self {
            notification_channel,
            message_id,
            progress: Percentage::new(4),
        }
    }

    pub fn build(self, sv_path: &Path) -> OombakGenResult<(TempGenDir, Probe)> {
        let source_paths: Vec<String> = source_paths_from_sv_path(sv_path)?
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        self.notify_progress("Creating probe...");
        let top_module_name = match sv_path.file_name().map(|f| f.to_string_lossy()) {
            Some(file_name) if file_name.ends_with(".sv") => {
                file_name.trim_end_matches(".sv").to_string()
            }
            Some(_) => return Err(Error::ExtensionNotSv(sv_path.to_path_buf())),
            None => return Err(Error::InvalidPath(sv_path.to_path_buf())),
        };
        let probe = Probe::try_from(&source_paths, &top_module_name)?;

        Ok((self.build_with_probe(sv_path, &probe)?, probe))
    }

    pub fn build_with_probe(
        mut self,
        sv_path: &Path,
        probe: &Probe,
    ) -> OombakGenResult<TempGenDir> {
        // Increment progress since Probe is already supplied
        self.progress.increment();

        self.notify_progress("Generating CMake project...");
        let source_dir = generator::generate(sv_path, probe)?;
        self.progress.increment();

        self.cmake(source_dir)
    }

    fn cmake(mut self, source_dir: TempDir) -> OombakGenResult<TempGenDir> {
        self.cmake_configure(source_dir.path())?;
        self.cmake_build(source_dir.path())?;
        self.notify_progress("liboombak.so generated!");
        let mut lib_path = PathBuf::from("build");
        lib_path.push("libdut.so");
        Ok(TempGenDir {
            tempdir: source_dir,
            lib_path,
        })
    }

    fn cmake_configure(&mut self, source_path: &Path) -> OombakGenResult<()> {
        self.notify_progress("Running CMake configure...");
        let output = Command::new("cmake")
            .current_dir(source_path)
            .args(["-S", ".", "-B", "build"])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CMake(stderr));
        }
        self.progress.increment();
        Ok(())
    }

    fn cmake_build(&mut self, source_path: &Path) -> OombakGenResult<()> {
        self.notify_progress("Running CMake build...");
        let output = Command::new("cmake")
            .current_dir(source_path)
            .args(["--build", "build"])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::CMake(stderr));
        }
        self.progress.increment();
        Ok(())
    }

    fn notify_progress(&self, message: &str) {
        if let Some(channel) = &self.notification_channel {
            let progress =
                oombak_sim::response::Payload::progress(message.to_string(), self.progress.clone());
            let response = oombak_sim::Message::response(self.message_id, progress);
            channel.blocking_send(response).unwrap();
        }
    }
}

fn source_paths_from_sv_path(sv_path: &Path) -> OombakGenResult<Vec<PathBuf>> {
    if !sv_path.exists() || !sv_path.is_file() {
        return Err(Error::SvFilePathNotFound(sv_path.to_path_buf()));
    }
    let mut source_paths = vec![];
    source_paths.push(sv_path.to_path_buf());
    let parent_dir = sv_path
        .parent()
        .ok_or(Error::InvalidPath(sv_path.to_path_buf()))?;
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

impl TempGenDir {
    pub fn lib_path(&self) -> PathBuf {
        self.tempdir.path().join(&self.lib_path)
    }
}
