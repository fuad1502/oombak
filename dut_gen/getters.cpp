#include "dut.hpp"

using namespace std;

vector<uint32_t> Dut::get_clk(Dut *self) {
  svBit out;
  self->vDut->v_sample_get_clk(&out);
  return vector<uint32_t>(1, out);
}
vector<uint32_t> Dut::get_rst_n(Dut *self) {
  svBit out;
  self->vDut->v_sample_get_rst_n(&out);
  return vector<uint32_t>(1, out);
}
vector<uint32_t> Dut::get_in(Dut *self) {
  int nBits = 6;
  svBitVecVal out[nBits / 32 + 1];
  self->vDut->v_sample_get_in(out);
  return Dut::get_vec_from(out, nBits);
}
vector<uint32_t> Dut::get_out(Dut *self) {
  int nBits = 6;
  svBitVecVal out[nBits / 32 + 1];
  self->vDut->v_sample_get_out(out);
  return Dut::get_vec_from(out, nBits);
}
vector<uint32_t> Dut::get_sample_DOT_c(Dut *self) {
  int nBits = 6;
  svBitVecVal out[nBits / 32 + 1];
  self->vDut->_ombak_get_sample_DOT_c(out);
  return Dut::get_vec_from(out, nBits);
}
vector<uint32_t> Dut::get_sample_DOT_adder_inst_DOT_d(Dut *self) {
  svBit out;
  self->vDut->_ombak_get_sample_DOT_adder_inst_DOT_d(&out);
  return vector<uint32_t>(1, out);
}
