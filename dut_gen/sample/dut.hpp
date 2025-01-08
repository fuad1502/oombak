#pragma once

#include "Vdut.h"
#include "verilated.h"
#include <cstdint>
#include <memory>
#include <optional>

class Dut;

struct Signal {
  std::optional<std::function<bool(Dut *, const std::vector<uint32_t> &)>> set;
  std::optional<
      std::function<std::pair<std::vector<uint32_t>, uint64_t>(Dut *)>>
      get;
};

class Dut {
public:
  Dut();
  ~Dut();
  std::optional<uint64_t> run(uint64_t duration);
  bool set(const std::string &sig_name, const std::vector<uint32_t> &words);
  std::optional<std::pair<std::vector<uint32_t>, uint64_t>>
  get(const std::string &sig_name);

private:
  std::unique_ptr<VerilatedContext> context;
  std::unique_ptr<Vdut> vDut;
  std::map<std::string, Signal> signalMapping;

  static std::vector<uint32_t> get_words_vec_from(svBitVecVal *out, int n_bits);
  static bool set_from_words_vec(svBitVecVal *in,
                                 const std::vector<uint32_t> &words,
                                 int n_bits);
  static void set_signal_mappings(std::map<std::string, Signal> &signalMapping);

  // setters
  static bool set_clk(Dut *self, const std::vector<uint32_t> &words);
  static bool set_rst_n(Dut *self, const std::vector<uint32_t> &words);
  static bool set_in(Dut *self, const std::vector<uint32_t> &words);
  // getters
  static std::pair<std::vector<uint32_t>, uint64_t> get_clk(Dut *self);
  static std::pair<std::vector<uint32_t>, uint64_t> get_rst_n(Dut *self);
  static std::pair<std::vector<uint32_t>, uint64_t> get_in(Dut *self);
  static std::pair<std::vector<uint32_t>, uint64_t> get_out(Dut *self);
  static std::pair<std::vector<uint32_t>, uint64_t> get_sample_DOT_c(Dut *self);
  static std::pair<std::vector<uint32_t>, uint64_t>
  get_sample_DOT_adder_inst_DOT_d(Dut *self);
};
