use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{error::DutGenResult, parser};

pub fn generate(sv_path: &Path, probe: &parser::Probe) -> DutGenResult<PathBuf> {
    Generator::new().generate(sv_path, probe)
}

struct Generator {
    temp_dir: PathBuf,
}

impl Generator {
    fn new() -> Self {
        Generator {
            temp_dir: PathBuf::new(),
        }
    }

    fn generate(&mut self, _sv_path: &Path, _probe: &parser::Probe) -> DutGenResult<PathBuf> {
        self.create_temp_dir()?;
        self.put_dut_bind_cpp()?;
        self.put_dut_bind_h()?;
        Ok(PathBuf::new())
    }

    fn create_temp_dir(&mut self) -> DutGenResult<()> {
        self.temp_dir = PathBuf::from("dut_gen_temp_dir");
        std::fs::create_dir(&self.temp_dir)?;
        Ok(())
    }

    fn put_dut_bind_cpp(&self) -> DutGenResult<()> {
        let mut file_path = self.temp_dir.clone();
        file_path.push("dut_bind.cpp");
        let mut file = File::create_new(file_path)?;
        let buf = include_bytes!("templates/dut_bind.cpp.fixed");
        file.write_all(buf)?;
        Ok(())
    }

    fn put_dut_bind_h(&self) -> DutGenResult<()> {
        let mut file_path = self.temp_dir.clone();
        file_path.push("dut_bind.hpp");
        let mut file = File::create_new(file_path)?;
        let buf = include_bytes!("templates/dut_bind.h.fixed");
        file.write_all(buf)?;
        Ok(())
    }
}
