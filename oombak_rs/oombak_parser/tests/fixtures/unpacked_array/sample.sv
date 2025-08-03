module sample #(
    int DLEN = 6
) (
    inout  logic [DLEN-1:0] a[1],
    inout  logic [DLEN-1:0] b[1],
    output logic [DLEN-1:0] c[1]
);

  wire d;

  assign {d, c[0]} = a[0] + b[0];

endmodule

