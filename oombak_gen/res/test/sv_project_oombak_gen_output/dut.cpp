#include "dut.hpp"
#include "svdpi.h"

using namespace std;

Dut::Dut() : context(new VerilatedContext), vDut(new Vdut) {
  vDut->eval();
  const svScope scope = svGetScopeFromName("TOP.dut");
  assert(scope);
  svSetScope(scope);
  Dut::set_signal_mappings(signalMapping);
}

Dut::~Dut() { vDut->final(); }

bool Dut::set(const std::string &sig_name, const std::vector<uint32_t> &words) {
  if (signalMapping.count(sig_name) == 0 ||
      !signalMapping[sig_name].set.has_value()) {
    return false;
  }
  return signalMapping[sig_name].set.value()(this, words);
}

std::optional<std::pair<std::vector<uint32_t>, uint64_t>>
Dut::get(const std::string &sig_name) {
  if (signalMapping.count(sig_name) == 0 ||
      !signalMapping[sig_name].get.has_value()) {
    return nullopt;
  }
  return signalMapping[sig_name].get.value()(this);
}

optional<uint64_t> Dut::run(uint64_t duration) {
  if (context->gotFinish()) {
    return nullopt;
  }
  if (vDut->eventsPending() &&
      context->time() + duration > vDut->nextTimeSlot()) {
    context->time(vDut->nextTimeSlot());
  } else {
    context->timeInc(duration);
  }
  vDut->eval();
  return context->time();
}

vector<uint32_t> Dut::get_words_vec_from(svBitVecVal *out, int n_bits) {
  vector<uint32_t> res;
  for (int i = 0; i < n_bits;) {
    svBitVecVal val;
    int w = min(32, n_bits - i);
    svGetPartselBit(&val, out, i, w);
    i += w;
    res.push_back(val);
  }
  return res;
}

bool Dut::set_from_words_vec(svBitVecVal *in, const vector<uint32_t> &words,
                             int n_bits) {
  if (words.size() * 32 < n_bits)
    return false;
  for (int i = 0; i < n_bits;) {
    svBitVec32 val = words[i / 32];
    int w = min(32, n_bits - i);
    svPutPartselBit(in, val, i, w);
    i += w;
  }
  return true;
}

void Dut::set_signal_mappings(std::map<std::string, Signal> &signalMapping) {
  // setters
  signalMapping["clk"].set = set_clk;
  signalMapping["rst_n"].set = set_rst_n;
  signalMapping["in"].set = set_in;
  // getters
  signalMapping["clk"].get = get_clk;
  signalMapping["rst_n"].get = get_rst_n;
  signalMapping["in"].get = get_in;
  signalMapping["out"].get = get_out;
  signalMapping["sample.c"].get = get_sample_DOT_c;
  signalMapping["sample.adder_inst.d"].get = get_sample_DOT_adder_inst_DOT_d;
}
