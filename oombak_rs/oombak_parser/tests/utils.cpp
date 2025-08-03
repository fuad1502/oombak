#include "utils.hpp"
#include <cstring>

bool operator==(const oombak_parser_signal_t &lhs, const oombak_parser_signal_t &rhs)
{
    return (strcmp(lhs.name, rhs.name) == 0) && (lhs.type == rhs.type) && (lhs.width == rhs.width);
}

std::ostream &operator<<(std::ostream &outs, const oombak_parser_signal_t &value)
{
    return outs << "{ " << value.name << ", " << value.type << ", " << value.width << " }";
}
