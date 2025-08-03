module sample #(
    int DLEN = 6
) (
    input logic clk,
    input logic rst_n,
    input logic [DLEN - 1:0] in,
    output logic [DLEN - 1:0] out
);
  wire [DLEN - 1:0] c;

  adder adder_inst (
      .a(in),
      .b(out),
      .c(c)
  );

  always_ff @(posedge clk) begin : counter
    if (!rst_n) begin
      out <= 0;
    end else begin
      out <= c;
    end
  end
endmodule
