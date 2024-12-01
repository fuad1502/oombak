#include "dut_bind.h"
#include "dut.hpp"

using namespace std;

static Dut dut;
static uint32_t *get_res;

char **query() { return NULL; }

RESULT set(char *sig_name, uint32_t *words, uint64_t len) {
  vector<uint32_t> v_words;
  for (int i = 0; i < len; i++)
    v_words.push_back(words[i]);
  if (dut.set(std::string(sig_name), v_words)) {
    return OK;
  } else {
    return ERR;
  }
}

uint32_t *get(char *sig_name, uint64_t *len) {
  auto res = dut.get(std::string(sig_name));
  if (!res.has_value()) {
    return NULL;
  }
  auto words_v = res.value().first;
  *len = res.value().second;
  free(get_res);
  get_res = (uint32_t *)malloc((*len / 32) + (*len % 32 != 0));
  for (int i = 0; i < words_v.size(); i++)
    get_res[i] = words_v[i];
  return get_res;
}

RESULT run(uint64_t duration, uint64_t *current_time) {
  auto res = dut.run(duration);
  if (!res.has_value()) {
    return ERR;
  }
  *current_time = res.value();
  return OK;
}
