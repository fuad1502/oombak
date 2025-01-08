#ifndef DUT_BIND_H
#define DUT_BIND_H

#include <cstdint>

typedef enum {
  OK = 0,
  ERR = -1,
} RESULT;

typedef struct {
  const char *name;
  uint64_t width;
  uint8_t get;
  uint8_t set;
} sig_t;

extern "C" sig_t *query(uint64_t *num_of_signals);
extern "C" RESULT set(char *sig_name, uint32_t *words, uint64_t num_of_words);
extern "C" uint32_t *get(char *sig_name, uint64_t *n_bits);
extern "C" RESULT run(uint64_t duration, uint64_t *current_time);

#endif // DUT_BIND_H
