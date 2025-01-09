use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{error::DutGenResult, parser};

macro_rules! generate_lines_from_name_template {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &crate::parser::Signal>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.name))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

macro_rules! generate_lines_from_name_width_template {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &crate::parser::Signal>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.name, s.width))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

macro_rules! single_bit_getter_template {
    () => {
        concat!(
            "pair<vector<uint32_t>, uint64_t> Dut::get_{0}(Dut *self) {{\n",
            "    svBit out;\n",
            "    self->vDut->v_sample_get_{0}(&out);\n",
            "    return {{vector<uint32_t>(1, out), 1}};\n",
            "}}\n"
        )
    };
}

macro_rules! single_bit_setter_template {
    () => {
        concat!(
            "bool Dut::set_{0}(Dut *self, const vector<uint32_t> &words) {{\n",
            "  if (words.size() > 0) {{\n",
            "    self->vDut->v_sample_set_{0}(words[0]);\n",
            "    return true;\n",
            "  }}\n",
            "  return false;\n",
            "}}\n"
        )
    };
}

macro_rules! multi_bit_getter_template {
    () => {
        concat!(
            "pair<vector<uint32_t>, uint64_t> Dut::get_{0}(Dut *self) {{\n",
            "  int nBits = {1};\n",
            "  svBitVecVal out[nBits / 32 + 1];\n",
            "  self->vDut->v_sample_get_{0}(out);\n",
            "  return {{Dut::get_words_vec_from(out, nBits), nBits}};\n",
            "}}\n"
        )
    };
}

macro_rules! multi_bit_setter_template {
    () => {
        concat!(
            "bool Dut::set_{0}(Dut *self, const vector<uint32_t> &words) {{\n",
            "  int nBits = {1};\n",
            "  svBitVecVal in[nBits / 32];\n",
            "  if (Dut::set_from_words_vec(in, words, nBits)) {{\n",
            "    self->vDut->v_sample_set_{0}(in);\n",
            "    return true;\n",
            "  }}\n",
            "  return false;\n",
            "}}\n",
        )
    };
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
        self.put_getters_cpp()?;
        self.put_setters_cpp()?;
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
        let setters = generate_lines_from_name_template!(
            "signalMapping[\"{0}\"].set = set_{0}",
            self.probe.signals.iter().filter(|s| s.set)
        );
        let getters = generate_lines_from_name_template!(
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
        let setters = generate_lines_from_name_template!(
            "static bool set_{0}(Dut *self, const std::vector<uint32_t> &words);",
            self.probe.signals.iter().filter(|s| s.set)
        );
        let getters = generate_lines_from_name_template!(
            "static std::pair<std::vector<uint32_t>, uint64_t> get_{0}(Dut *self);",
            self.probe.signals.iter().filter(|s| s.get)
        );
        let content = content.replace("// TEMPLATED: setters", &setters);
        let content = content.replace("// TEMPLATED: getters", &getters);
        self.put_file("dut.hpp", content.as_bytes())?;
        Ok(())
    }

    fn put_getters_cpp(&self) -> DutGenResult<()> {
        let content = include_str!("templates/getters.cpp.templated");
        let single_bit_signals = self.probe.signals.iter().filter(|s| s.get && s.width == 1);
        let multi_bit_signals = self.probe.signals.iter().filter(|s| s.get && s.width > 1);
        let single_bit_getters =
            generate_lines_from_name_template!(single_bit_getter_template!(), single_bit_signals);
        let multi_bit_getters = generate_lines_from_name_width_template!(
            multi_bit_getter_template!(),
            multi_bit_signals
        );
        let content = content.replace(
            "// TEMPLATED: getters",
            &(single_bit_getters + "\n" + &multi_bit_getters),
        );
        self.put_file("getters.cpp", content.as_bytes())?;
        Ok(())
    }

    fn put_setters_cpp(&self) -> DutGenResult<()> {
        let content = include_str!("templates/setters.cpp.templated");
        let single_bit_signals = self.probe.signals.iter().filter(|s| s.set && s.width == 1);
        let multi_bit_signals = self.probe.signals.iter().filter(|s| s.set && s.width > 1);
        let single_bit_setters =
            generate_lines_from_name_template!(single_bit_setter_template!(), single_bit_signals);
        let multi_bit_setters = generate_lines_from_name_width_template!(
            multi_bit_setter_template!(),
            multi_bit_signals
        );
        let content = content.replace(
            "// TEMPLATED: setters",
            &(single_bit_setters + "\n" + &multi_bit_setters),
        );
        self.put_file("setters.cpp", content.as_bytes())?;
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
