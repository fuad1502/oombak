use std::path::Path;

pub struct Probe {
    pub signals: Vec<Signal>,
}

pub struct Signal {
    pub name: String,
    pub width: u64,
    pub get: bool,
    pub set: bool,
}

pub fn parse(_sv_path: &Path) -> Result<Probe, String> {
    // temporarility return sample dut signals
    let signals = vec![
        Signal {
            name: "clk".to_string(),
            width: 1,
            get: true,
            set: true,
        },
        Signal {
            name: "rst_n".to_string(),
            width: 1,
            get: true,
            set: true,
        },
        Signal {
            name: "in".to_string(),
            width: 6,
            get: true,
            set: true,
        },
        Signal {
            name: "out".to_string(),
            width: 6,
            get: true,
            set: false,
        },
        Signal {
            name: "sample.c".to_string(),
            width: 6,
            get: true,
            set: false,
        },
        Signal {
            name: "sample.adder_inst.d".to_string(),
            width: 1,
            get: true,
            set: false,
        },
    ];
    Ok(Probe { signals })
}
