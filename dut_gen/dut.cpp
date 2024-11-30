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

bool Dut::set(const std::string &sig_name, const std::vector<uint32_t> &bytes) {
  if (signalMapping.count(sig_name) == 0 ||
      !signalMapping[sig_name].set.has_value()) {
    return false;
  }
  signalMapping[sig_name].set.value()(this, bytes);
  return true;
}

std::optional<std::vector<uint32_t>> Dut::get(const std::string &sig_name) {
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

vector<uint32_t> Dut::get_vec_from(svBitVecVal *out, int nBits) {
  vector<uint32_t> res;
  for (int i = 0; i < nBits;) {
    svBitVecVal val;
    int w = min(32, nBits - i);
    svGetPartselBit(&val, out, i, w);
    i += w;
    res.push_back(val);
  }
  return res;
}

void Dut::set_from_vec(svBitVecVal *in, const vector<uint32_t> &bytes,
                       int nBits) {
  for (int i = 0; i < nBits;) {
    svBitVec32 val = bytes[i / 32];
    int w = min(32, nBits - i);
    svPutPartselBit(in, val, i, w);
    i += w;
  }
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
}
