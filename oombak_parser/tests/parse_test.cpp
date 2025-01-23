#include "oombak_parser.h"
#include "utils.hpp"
#include "gtest/gtest.h"

TEST(ParseTest, SvSample1) {
  const char *source_paths =
      "fixtures/sv_sample_1/sample.sv:fixtures/sv_sample_1/adder.sv";
  const char *top_module_name = "sample";
  auto root_instance = oombak_parser_parse(source_paths, top_module_name);

  ASSERT_NE(root_instance, (Instance *)NULL);
  EXPECT_STREQ(root_instance->name, "sample");
  EXPECT_STREQ(root_instance->module_name, "sample");
  EXPECT_EQ(root_instance->parent_instance, (Instance *)NULL);

  Signal expected_signals[] = {{"clk", UnpackedArrPortIn, 1},
                               {"rst_n", UnpackedArrPortIn, 1},
                               {"in", UnpackedArrPortIn, 6},
                               {"out", UnpackedArrPortOut, 6},
                               {"c", UnpackedArrVarNet, 6}};
  EXPECT_EQ(root_instance->signals_len, 5);
  EXPECT_TRUE(isContainsAll(root_instance->signals, root_instance->signals_len,
                            expected_signals, 5));

  ASSERT_EQ(root_instance->child_instances_len, 1);
  auto child_instance = root_instance->child_instances[0];
  ASSERT_EQ(child_instance->parent_instance, root_instance);
  ASSERT_STREQ(child_instance->name, "adder_inst");
  ASSERT_STREQ(child_instance->module_name, "adder");
  ASSERT_EQ(child_instance->child_instances_len, 0);
  ASSERT_EQ(child_instance->signals_len, 4);
}
