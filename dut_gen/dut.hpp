#pragma once

#include "Vdut.h"
#include "verilated.h"
#include <cstdint>
#include <memory>
#include <optional>

class Dut;

struct Signal {
  std::optional<std::function<void(Dut *, const std::vector<uint32_t> &)>> set;
  std::optional<std::function<std::vector<uint32_t>(Dut *)>> get;
};

class Dut {
public:
  Dut();
  ~Dut();
  std::optional<uint64_t> run(uint64_t duration);
  bool set(const std::string &sig_name, const std::vector<uint32_t> &bytes);
  std::optional<std::vector<uint32_t>> get(const std::string &sig_name);

private:
  std::unique_ptr<VerilatedContext> context;
  std::unique_ptr<Vdut> vDut;
  std::map<std::string, Signal> signalMapping;

  static std::vector<uint32_t> get_vec_from(svBitVecVal *out, int nBits);
  static void set_from_vec(svBitVecVal *in, const std::vector<uint32_t> &bytes,
                           int nBits);
  static void set_signal_mappings(std::map<std::string, Signal> &signalMapping);

  // setters
  static void set_clk(Dut *self, const std::vector<uint32_t> &bytes);
  static void set_rst_n(Dut *self, const std::vector<uint32_t> &bytes);
  static void set_in(Dut *self, const std::vector<uint32_t> &bytes);
  // getters
  static std::vector<uint32_t> get_clk(Dut *self);
  static std::vector<uint32_t> get_rst_n(Dut *self);
  static std::vector<uint32_t> get_in(Dut *self);
  static std::vector<uint32_t> get_out(Dut *self);
};
