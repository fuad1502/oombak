module dut ();
  logic clk;
  logic rst_n;
  logic [5:0] in;
  wire [5:0] out;

  sample sample_inst (
      .clk(clk),
      .rst_n(rst_n),
      .in(in),
      .out(out)
  );

  // setters
  export "DPI-C" function v_sample_set_clk;
  function automatic void v_sample_set_clk(input bit _clk);
    clk = _clk;
  endfunction
  export "DPI-C" function v_sample_set_rst_n;
  function automatic void v_sample_set_rst_n(input bit _rst_n);
    rst_n = _rst_n;
  endfunction
  export "DPI-C" function v_sample_set_in;
  function automatic void v_sample_set_in(input bit [5:0] _in);
    in = _in;
  endfunction

  // getters
  export "DPI-C" function v_sample_get_clk;
  function automatic void v_sample_get_clk(output bit _clk);
    _clk = clk;
  endfunction
  export "DPI-C" function v_sample_get_rst_n;
  function automatic void v_sample_get_rst_n(output bit _rst_n);
    _rst_n = rst_n;
  endfunction
  export "DPI-C" function v_sample_get_in;
  function automatic void v_sample_get_in(output bit [5:0] _in);
    _in = in;
  endfunction
  export "DPI-C" function v_sample_get_out;
  function automatic v_sample_get_out(output bit [5:0] _out);
    _out = out;
  endfunction

endmodule
