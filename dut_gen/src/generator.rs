use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{error::DutGenResult, parser};

macro_rules! generate_lines_from_template {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &crate::parser::Signal>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.name))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

pub fn generate(_sv_path: &Path, probe: &parser::Probe) -> DutGenResult<PathBuf> {
    Generator::new(probe).generate()
}

struct Generator<'a> {
    temp_dir: PathBuf,
    probe: &'a parser::Probe,
}

impl<'a> Generator<'a> {
    fn new(probe: &'a parser::Probe) -> Self {
        Generator {
            temp_dir: PathBuf::new(),
            probe,
        }
    }

    fn generate(&mut self) -> DutGenResult<PathBuf> {
        self.create_temp_dir()?;
        self.put_dut_bind_cpp()?;
        self.put_dut_bind_h()?;
        self.put_dut_cpp()?;
        self.put_dut_hpp()?;
        Ok(PathBuf::new())
    }

    fn create_temp_dir(&mut self) -> DutGenResult<()> {
        self.temp_dir = PathBuf::from("dut_gen_temp_dir");
        std::fs::create_dir(&self.temp_dir)?;
        Ok(())
    }

    fn put_dut_bind_cpp(&self) -> DutGenResult<()> {
        let content = include_bytes!("templates/dut_bind.cpp.fixed");
        self.put_file("dut_bind.cpp", content)?;
        Ok(())
    }

    fn put_dut_bind_h(&self) -> DutGenResult<()> {
        let content = include_bytes!("templates/dut_bind.h.fixed");
        self.put_file("dut_bind.h", content)?;
        Ok(())
    }

    fn put_dut_cpp(&self) -> DutGenResult<()> {
        let content = include_str!("templates/dut.cpp.templated");
        let setters = generate_lines_from_template!(
            "signalMapping[\"{0}\"].set = set_{0}",
            self.probe.signals.iter().filter(|s| s.set)
        );
        let getters = generate_lines_from_template!(
            "signalMapping[\"{0}\"].get = get_{0}",
            self.probe.signals.iter().filter(|s| s.get)
        );
        let content = content.replace("// TEMPLATED: setters", &setters);
        let content = content.replace("// TEMPLATED: getters", &getters);
        self.put_file("dut.cpp", content.as_bytes())?;
        Ok(())
    }

    fn put_dut_hpp(&self) -> DutGenResult<()> {
        let content = include_str!("templates/dut.hpp.templated");
        let setters = generate_lines_from_template!(
            "static bool set_{0}(Dut *self, const std::vector<uint32_t> &words);",
            self.probe.signals.iter().filter(|s| s.set)
        );
        let getters = generate_lines_from_template!(
            "static std::pair<std::vector<uint32_t>, uint64_t> get_{0}(Dut *self);",
            self.probe.signals.iter().filter(|s| s.get)
        );
        let content = content.replace("// TEMPLATED: setters", &setters);
        let content = content.replace("// TEMPLATED: getters", &getters);
        self.put_file("dut.hpp", content.as_bytes())?;
        Ok(())
    }

    fn put_file(&self, file_name: &str, content: &[u8]) -> DutGenResult<()> {
        let mut file_path = self.temp_dir.clone();
        file_path.push(file_name);
        let mut file = File::create_new(file_path)?;
        file.write_all(content)?;
        Ok(())
    }
}
