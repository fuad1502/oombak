#include "dut.hpp"

using namespace std;

void Dut::set_clk(Dut *self, const vector<uint32_t> &bytes) {
  self->vDut->v_sample_set_clk(bytes[0]);
}
void Dut::set_rst_n(Dut *self, const vector<uint32_t> &bytes) {
  self->vDut->v_sample_set_rst_n(bytes[0]);
}
void Dut::set_in(Dut *self, const vector<uint32_t> &bytes) {
  int nBits = 6;
  svBitVecVal in[nBits / 32];
  Dut::set_from_vec(in, bytes, nBits);
  self->vDut->v_sample_set_in(in);
}
