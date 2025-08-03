#pragma once

#include "oombak_parser.h"
#include "gtest/gtest.h"
#include <cstdint>

using testing::AssertionFailure;
using testing::AssertionResult;
using testing::AssertionSuccess;

bool operator==(const oombak_parser_signal_t &lhs,
                const oombak_parser_signal_t &rhs);

std::ostream &operator<<(std::ostream &outs,
                         const oombak_parser_signal_t &value);

template <typename T>
AssertionResult isContainsAll(T *value, uint64_t value_len, T *expected,
                              uint64_t expected_len) {
  if (value_len != expected_len) {
    return AssertionFailure() << value_len << " != " << expected_len;
  }
  for (auto i = 0; i < expected_len; i++) {
    bool isExist = false;
    for (auto j = 0; j < value_len; j++) {
      if (expected[i] == value[j]) {
        isExist = true;
      }
    }
    if (!isExist) {
      return AssertionFailure() << expected[i] << " does not exists in value";
    }
  }
  return AssertionSuccess();
}
