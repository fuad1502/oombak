#include "dut_bind.h"

uint64_t num_of_signals = 6;

sig_t signals[6] = {
    {"clk", 6, 1, 1}, {"rst_n", 1, 1, 1},    {"in", 6, 1, 1},
    {"out", 6, 1, 0}, {"sample.c", 6, 1, 0}, {"sample.adder_inst.d", 1, 1, 0},
};
