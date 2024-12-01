#ifndef DUT_BIND_H
#define DUT_BIND_H

#include <cstdint>

typedef enum {
  OK = 0,
  ERR = -1,
} RESULT;

extern "C" char **query();
extern "C" RESULT set(char *sig_name, uint32_t *words, uint64_t len);
extern "C" uint32_t *get(char *sig_name, uint64_t *n_bits);
extern "C" RESULT run(uint64_t duration, uint64_t *current_time);

#endif // DUT_BIND_H
