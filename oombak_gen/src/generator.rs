use std::{fs::File, io::Write, path::Path};

use crate::{Error, OombakGenResult};

use oombak_rs::probe::{Probe, ProbePoint};
use tempfile::TempDir;

macro_rules! generate_lines_from_name_template {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &ProbePoint>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.get_dot_replaced_path()))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

macro_rules! generate_lines_from_name_width_template {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &ProbePoint>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.get_dot_replaced_path(), s.bit_width()))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

macro_rules! generate_lines_from_dot_replaced_name_name {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &ProbePoint>> = Box::new($signals);
        signals
            .map(|s| format!($template, s.get_dot_replaced_path(), s.path()))
            .collect::<Vec<String>>()
            .join("\n")
    }};
}

macro_rules! generate_lines_from_dot_replaced_name_name_width {
    ($template:expr, $signals:expr) => {{
        let signals: Box<dyn Iterator<Item = &ProbePoint>> = Box::new($signals);
        signals
            .map(|s| {
                format!(
                    $template,
                    s.get_dot_replaced_path(),
                    s.path(),
                    s.bit_width() - 1
                )
            })
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

macro_rules! single_bit_dpc_setter_template {
    () => {
        concat!(
            "export \"DPI-C\" function v_sample_set_{0};\n",
            "function automatic void v_sample_set_{0}(input bit _in);\n",
            "  {1} = _in;\n",
            "endfunction\n"
        )
    };
}

macro_rules! multi_bit_dpc_setter_template {
    () => {
        concat!(
            "export \"DPI-C\" function v_sample_set_{0};\n",
            "function automatic void v_sample_set_{0}(input bit [{2}:0] _in);\n",
            "  {1} = _in;\n",
            "endfunction\n"
        )
    };
}

macro_rules! single_bit_dpc_getter_template {
    () => {
        concat!(
            "export \"DPI-C\" function v_sample_get_{0};\n",
            "function automatic void v_sample_get_{0}(output bit _out);\n",
            "  _out = {1};\n",
            "endfunction\n"
        )
    };
}

macro_rules! multi_bit_dpc_getter_template {
    () => {
        concat!(
            "export \"DPI-C\" function v_sample_get_{0};\n",
            "function automatic void v_sample_get_{0}(output bit [{2}:0] _out);\n",
            "  _out = {1};\n",
            "endfunction\n"
        )
    };
}

pub fn generate(sv_path: &Path, probe: &Probe) -> OombakGenResult<TempDir> {
    Generator::new(probe, sv_path)?.generate()
}

struct Generator<'a> {
    temp_dir: TempDir,
    probe: &'a Probe,
    sv_path: &'a Path,
}

impl<'a> Generator<'a> {
    fn new(probe: &'a Probe, sv_path: &'a Path) -> OombakGenResult<Self> {
        Ok(Generator {
            temp_dir: TempDir::new()?,
            probe,
            sv_path,
        })
    }

    fn generate(self) -> OombakGenResult<TempDir> {
        self.put_dut_bind_cpp()?;
        self.put_dut_bind_h()?;
        self.put_dut_cpp()?;
        self.put_dut_hpp()?;
        self.put_getters_cpp()?;
        self.put_setters_cpp()?;
        self.put_signals_cpp()?;
        self.put_ombak_dut_sv()?;
        self.put_cmakelists_txt()?;
        Ok(self.temp_dir)
    }

    fn put_dut_bind_cpp(&self) -> OombakGenResult<()> {
        let content = include_bytes!("templates/dut_bind.cpp.fixed");
        self.put_file("dut_bind.cpp", content)?;
        Ok(())
    }

    fn put_dut_bind_h(&self) -> OombakGenResult<()> {
        let content = include_bytes!("templates/dut_bind.h.fixed");
        self.put_file("dut_bind.h", content)?;
        Ok(())
    }

    fn put_dut_cpp(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/dut.cpp.templated");
        let setters = generate_lines_from_dot_replaced_name_name!(
            "signalMapping[\"{1}\"].set = set_{0};",
            self.probe.get_settable_points()
        );
        let getters = generate_lines_from_dot_replaced_name_name!(
            "signalMapping[\"{1}\"].get = get_{0};",
            self.probe.get_gettable_points()
        );
        let content = content.replace("// TEMPLATED: setters", &setters);
        let content = content.replace("// TEMPLATED: getters", &getters);
        self.put_file("dut.cpp", content.as_bytes())?;
        Ok(())
    }

    fn put_dut_hpp(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/dut.hpp.templated");
        let setters = generate_lines_from_name_template!(
            "static bool set_{0}(Dut *self, const std::vector<uint32_t> &words);",
            self.probe.get_settable_points()
        );
        let getters = generate_lines_from_name_template!(
            "static std::pair<std::vector<uint32_t>, uint64_t> get_{0}(Dut *self);",
            self.probe.get_gettable_points()
        );
        let content = content.replace("// TEMPLATED: setters", &setters);
        let content = content.replace("// TEMPLATED: getters", &getters);
        self.put_file("dut.hpp", content.as_bytes())?;
        Ok(())
    }

    fn put_getters_cpp(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/getters.cpp.templated");
        let single_bit_signals = self.probe.get_single_bit_gettable_points();
        let multi_bit_signals = self.probe.get_multibit_gettable_points();
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

    fn put_setters_cpp(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/setters.cpp.templated");
        let single_bit_signals = self.probe.get_single_bit_settable_points();
        let multi_bit_signals = self.probe.get_multibit_settable_points();
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

    fn put_signals_cpp(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/signals.cpp.templated");
        let content = content.replace(
            "// TEMPLATED: num_of_signals",
            &format!("{};", &self.probe.get_probed_points().len()),
        );
        let signals = self.generate_signals_array();
        let content = content.replace("// TEMPLATED: signals", &signals);
        self.put_file("signals.cpp", content.as_bytes())?;
        Ok(())
    }

    fn put_ombak_dut_sv(&self) -> OombakGenResult<()> {
        let content = include_str!("templates/ombak_dut.sv.templated");
        let top_level_signal_declarations = self.generate_top_level_signal_declarations();
        let top_level_module_instantiation = self.generate_top_level_module_instantiation();
        let dpc_setters = self.generate_dpc_setters();
        let dpc_getters = self.generate_dpc_getters();
        let content = content.replace("// TEMPLATED: signals", &top_level_signal_declarations);
        let content = content.replace("// TEMPLATED: dut", &top_level_module_instantiation);
        let content = content.replace("// TEMPLATED: setters", &dpc_setters);
        let content = content.replace("// TEMPLATED: getters", &dpc_getters);
        self.put_file("ombak_dut.sv", content.as_bytes())?;
        Ok(())
    }

    fn put_cmakelists_txt(&self) -> OombakGenResult<()> {
        let sv_dir = self
            .sv_path
            .parent()
            .ok_or(Error::InvalidPath(self.sv_path.to_path_buf()))?;
        let content = include_str!("templates/CMakeLists.txt.templated");
        let content = content.replace(
            "/*OMBAK_INCLUDE_DIRS*/",
            sv_dir
                .to_str()
                .ok_or(Error::InvalidPath(self.sv_path.to_path_buf()))?,
        );
        self.put_file("CMakeLists.txt", content.as_bytes())?;
        Ok(())
    }

    fn generate_signals_array(&self) -> String {
        let num_of_signals = self.probe.get_probed_points().len();
        let mut signals_array = format!("oombak_sig_t signals[{num_of_signals}] = {{\n");
        for point in self.probe.get_probed_points() {
            let get = if point.is_gettable() { 1 } else { 0 };
            let set = if point.is_settable() { 1 } else { 0 };
            let width = point.bit_width();
            signals_array += &format!(
                "    {{ \"{}\", {}, {}, {} }},\n",
                point.path(),
                width,
                get,
                set
            );
        }
        signals_array += "};";
        signals_array
    }

    fn generate_top_level_signal_declarations(&self) -> String {
        self.probe
            .get_top_level_ports()
            .iter()
            .fold(String::from(""), |prev, p| {
                let width = if p.bit_width() > 1 {
                    format!("[{}:0]", p.bit_width() - 1)
                } else {
                    "".to_string()
                };
                prev + &format!("logic {width} {};\n", p.path())
            })
    }

    fn generate_top_level_module_instantiation(&self) -> String {
        let pin_assignments = self
            .probe
            .get_top_level_ports()
            .iter()
            .fold(String::from(""), |prev, p| {
                prev + &format!(".{0}({0}),\n", p.path())
            });
        format!(
            "{0} {0} (\n{1}\n);",
            self.probe.top_level_module_name(),
            &pin_assignments[..pin_assignments.len() - 2]
        )
    }

    fn generate_dpc_setters(&self) -> String {
        let single_bit_signals = self.probe.get_single_bit_settable_points();
        let multi_bit_signals = self.probe.get_multibit_settable_points();
        let single_bit_setters = generate_lines_from_dot_replaced_name_name!(
            single_bit_dpc_setter_template!(),
            single_bit_signals
        );
        let multi_bit_setters = generate_lines_from_dot_replaced_name_name_width!(
            multi_bit_dpc_setter_template!(),
            multi_bit_signals
        );
        single_bit_setters + &multi_bit_setters
    }

    fn generate_dpc_getters(&self) -> String {
        let single_bit_signals = self.probe.get_single_bit_gettable_points();
        let multi_bit_signals = self.probe.get_multibit_gettable_points();
        let single_bit_getters = generate_lines_from_dot_replaced_name_name!(
            single_bit_dpc_getter_template!(),
            single_bit_signals
        );
        let multi_bit_getters = generate_lines_from_dot_replaced_name_name_width!(
            multi_bit_dpc_getter_template!(),
            multi_bit_signals
        );
        single_bit_getters + &multi_bit_getters
    }

    fn put_file(&self, file_name: &str, content: &[u8]) -> OombakGenResult<()> {
        let file_path = self.temp_dir.path().join(file_name);
        let mut file = File::create_new(file_path)?;
        file.write_all(content)?;
        Ok(())
    }
}
