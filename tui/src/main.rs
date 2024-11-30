use std::path::PathBuf;

use tui::Dut;

fn main() {
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.push("../dut_gen/build/libdut.so");
    let lib_path = lib_path.to_string_lossy().to_string();

    let dut = Dut::new(&lib_path).unwrap();

    dut.set("in", vec![2, 0, 0, 0]).unwrap();
    dut.set("rst_n", vec![1]).unwrap();
    for _i in 1..5 {
        dut.set("clk", vec![0]).unwrap();
        dut.run(1).unwrap();
        dut.set("clk", vec![1]).unwrap();
        dut.run(1).unwrap();
        println!("{:?}", dut.get("out").unwrap());
    }
}
