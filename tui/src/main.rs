use bitvec::vec::BitVec;
use std::path::PathBuf;

use tui::Dut;

fn main() {
    // get lib path
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.push("../dut_gen/build/libdut.so");
    let lib_path = lib_path.to_string_lossy().to_string();

    let dut = Dut::new(&lib_path).unwrap();

    // Query available signals
    println!("{:?}", dut.query().unwrap());
    // Setup input signals
    dut.set("in", &BitVec::from_slice(&[16])).unwrap();
    dut.set("rst_n", &BitVec::from_slice(&[1])).unwrap();
    for _i in 1..5 {
        // Toggle clock
        dut.set("clk", &BitVec::from_slice(&[0])).unwrap();
        dut.run(1).unwrap();
        dut.set("clk", &BitVec::from_slice(&[1])).unwrap();
        dut.run(1).unwrap();
        // Monitor output
        println!("in = {}", dut.get("in").unwrap());
        println!("out = {}", dut.get("out").unwrap());
        println!("sample.c = {}", dut.get("sample.c").unwrap());
        println!(
            "sample.adder_inst.d = {}",
            dut.get("sample.adder_inst.d").unwrap()
        );
    }
}
