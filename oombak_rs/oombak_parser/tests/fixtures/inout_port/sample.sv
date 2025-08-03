module sample #(
    int DLEN = 6
) (
    inout  logic [DLEN-1:0] a,
    inout  logic [DLEN-1:0] b,
    output logic [DLEN-1:0] c
);

  wire d;

  assign {d, c} = a + b;

endmodule
