#include "dut.hpp"

using namespace std;

void Dut::set_clk(Dut *self, const vector<uint32_t> &words) {
  self->vDut->v_sample_set_clk(words[0]);
}
void Dut::set_rst_n(Dut *self, const vector<uint32_t> &words) {
  self->vDut->v_sample_set_rst_n(words[0]);
}
void Dut::set_in(Dut *self, const vector<uint32_t> &words) {
  int nBits = 6;
  svBitVecVal in[nBits / 32];
  Dut::set_from_words_vec(in, words, nBits);
  self->vDut->v_sample_set_in(in);
}
