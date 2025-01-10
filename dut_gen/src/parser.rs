use std::path::Path;

pub struct Probe {
    pub module_name: String,
    pub signals: Vec<Signal>,
}

pub struct Signal {
    pub name: String,
    pub width: u64,
    pub get: bool,
    pub set: bool,
    pub top_level: bool,
}

pub fn parse(_sv_path: &Path) -> Result<Probe, String> {
    // temporarility return sample dut signals
    let signals = vec![
        Signal {
            name: "clk".to_string(),
            width: 1,
            get: true,
            set: true,
            top_level: true,
        },
        Signal {
            name: "rst_n".to_string(),
            width: 1,
            get: true,
            set: true,
            top_level: true,
        },
        Signal {
            name: "in".to_string(),
            width: 6,
            get: true,
            set: true,
            top_level: true,
        },
        Signal {
            name: "out".to_string(),
            width: 6,
            get: true,
            set: false,
            top_level: true,
        },
        Signal {
            name: "sample_DOT_c".to_string(),
            width: 6,
            get: true,
            set: false,
            top_level: false,
        },
        Signal {
            name: "sample_DOT_adder_inst_DOT_d".to_string(),
            width: 1,
            get: true,
            set: false,
            top_level: false,
        },
    ];
    Ok(Probe {
        module_name: "sample".to_string(),
        signals,
    })
}
