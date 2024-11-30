#include "dut_bind.h"
#include "dut.hpp"

using namespace std;

static Dut dut;

char **query() { return NULL; }

RESULT set(char *sig_name, uint32_t *bytes, uint64_t len) {
  vector<uint32_t> v_bytes;
  for (int i = 0; i < len; i++) v_bytes.push_back(bytes[i]);
  if (dut.set(std::string(sig_name), v_bytes)) {
    return OK;
  } else {
    return ERR;
  }
}

RESULT get(char *sig_name, uint32_t *bytes, uint64_t len) {
  auto res = dut.get(std::string(sig_name));
  if (!res.has_value()) {
    return ERR;
  }
  auto bytes_v = res.value();
  for (int i = 0; i < len && i < bytes_v.size(); i++) bytes[i] = bytes_v[i];
  return OK;
}

RESULT run(uint64_t duration, uint64_t *current_time) {
  auto res = dut.run(duration);
  if (!res.has_value()) {
    return ERR;
  }
  *current_time = res.value();
  return OK;
}
