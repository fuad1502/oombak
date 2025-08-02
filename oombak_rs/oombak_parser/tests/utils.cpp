#include "utils.hpp"
#include <cstring>

bool operator==(const Signal &lhs, const Signal &rhs) {
  return (strcmp(lhs.name, rhs.name) == 0) && (lhs.type == rhs.type) &&
         (lhs.width == rhs.width);
}

std::ostream &operator<<(std::ostream &outs, const Signal &value) {
  return outs << "{ " << value.name << ", " << value.type << ", " << value.width
              << " }";
}
