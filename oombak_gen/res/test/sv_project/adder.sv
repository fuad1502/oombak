module adder #(
    int DLEN = 6
) (
    input  logic [DLEN-1:0] a,
    input  logic [DLEN-1:0] b,
    output logic [DLEN-1:0] c
);

wire d;

assign {d, c} = a + b;

endmodule
