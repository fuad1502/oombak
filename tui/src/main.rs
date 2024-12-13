use bitvec::vec::BitVec;
use std::path::PathBuf;

use tui::dut::Dut;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get lib path
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.push("../dut_gen/build/libdut.so");
    let lib_path = lib_path.to_string_lossy().to_string();

    let dut = Dut::new(&lib_path)?;

    // Query available signals
    println!("{:?}", dut.query()?);
    // Setup input signals
    dut.set("in", &BitVec::from_slice(&[16]))?;
    dut.set("rst_n", &BitVec::from_slice(&[1]))?;
    for _i in 1..5 {
        // Toggle clock
        dut.set("clk", &BitVec::from_slice(&[0]))?;
        dut.run(1).unwrap();
        dut.set("clk", &BitVec::from_slice(&[1]))?;
        dut.run(1).unwrap();
        // Monitor output
        println!("in = {}", dut.get("in")?);
        println!("out = {}", dut.get("out")?);
        println!("sample.c = {}", dut.get("sample.c")?);
        println!("sample.adder_inst.d = {}", dut.get("sample.adder_inst.d")?);
    }
    Ok(())
}
