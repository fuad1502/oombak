#include "parse_test.h"
#include "oombak_parser.h"
#include "utils.hpp"
#include "gtest/gtest.h"
#include <future>

TEST(ParseTest, SvSample1_Root)
{
    const char *source_paths = "fixtures/sv_sample_1/sample.sv:fixtures/sv_sample_1/adder.sv:fixtures/"
                               "sv_sample_1/subtractor.sv";
    const char *top_module_name = "sample";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_FALSE(result.is_error) << "oombak_parser_parse returned error code: " << result.error;
    auto root_instance = result.instance;

    ASSERT_NE(root_instance, (oombak_parser_instance_t *)NULL);
    EXPECT_STREQ(root_instance->name, "sample");
    EXPECT_STREQ(root_instance->module_name, "sample");
    EXPECT_EQ(root_instance->parent_instance, (oombak_parser_instance_t *)NULL);

    oombak_parser_signal_t expected_signals[] = {{"clk", OOMBAK_PARSER_PACKED_ARR_PORT_IN, 1},
                                                 {"rst_n", OOMBAK_PARSER_PACKED_ARR_PORT_IN, 1},
                                                 {"in", OOMBAK_PARSER_PACKED_ARR_PORT_IN, 6},
                                                 {"out", OOMBAK_PARSER_PACKED_ARR_PORT_OUT, 6},
                                                 {"c", OOMBAK_PARSER_PACKED_ARR_VAR_NET, 6}};
    EXPECT_EQ(root_instance->signals_len, 5);
    EXPECT_TRUE(isContainsAll(root_instance->signals, root_instance->signals_len, expected_signals, 5));

    ASSERT_EQ(root_instance->child_instances_len, 1);
    auto child_instance = root_instance->child_instances[0];
    ASSERT_EQ(child_instance->parent_instance, root_instance);
    ASSERT_STREQ(child_instance->name, "adder_inst");
    ASSERT_STREQ(child_instance->module_name, "adder");
    ASSERT_EQ(child_instance->child_instances_len, 0);
    ASSERT_EQ(child_instance->signals_len, 4);
}

TEST(ParseTest, SvSample1_NotRoot)
{
    const char *source_paths = "fixtures/sv_sample_1/sample.sv:fixtures/sv_sample_1/adder.sv:fixtures/"
                               "sv_sample_1/subtractor.sv";
    const char *top_module_name = "adder";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_FALSE(result.is_error) << "oombak_parser_parse returned error code: " << result.error;
    auto root_instance = result.instance;

    ASSERT_NE(root_instance, (oombak_parser_instance_t *)NULL);
    EXPECT_STREQ(root_instance->name, "adder");
    EXPECT_STREQ(root_instance->module_name, "adder");
    EXPECT_EQ(root_instance->parent_instance, (oombak_parser_instance_t *)NULL);

    oombak_parser_signal_t expected_signals[] = {{"a", OOMBAK_PARSER_PACKED_ARR_PORT_IN, 6},
                                                 {"b", OOMBAK_PARSER_PACKED_ARR_PORT_IN, 6},
                                                 {"c", OOMBAK_PARSER_PACKED_ARR_PORT_OUT, 6},
                                                 {"d", OOMBAK_PARSER_PACKED_ARR_VAR_NET, 1}};
    EXPECT_EQ(root_instance->signals_len, 4);
    EXPECT_TRUE(isContainsAll(root_instance->signals, root_instance->signals_len, expected_signals, 4));

    ASSERT_EQ(root_instance->child_instances_len, 0);
}

TEST(ParseTest, SvSample1_InvalidModule)
{
    const char *source_paths = "fixtures/sv_sample_1/sample.sv:fixtures/sv_sample_1/adder.sv:fixtures/"
                               "sv_sample_1/subtractor.sv";
    const char *top_module_name = "invalid_module";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_TRUE(result.is_error);
    ASSERT_EQ(result.error, OOMBAK_PARSER_ERROR_TOP_MODULE_NOT_FOUND);
}

TEST(ParseTest, SyntaxError)
{
    const char *source_paths = "fixtures/syntax_error/sample.sv";
    const char *top_module_name = "sample";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_TRUE(result.is_error);
    ASSERT_EQ(result.error, OOMBAK_PARSER_ERROR_COMPILE_ERROR);
}

TEST(ParseTest, InoutPort)
{
    const char *source_paths = "fixtures/inout_port/sample.sv";
    const char *top_module_name = "sample";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_TRUE(result.is_error);
    ASSERT_EQ(result.error, OOMBAK_PARSER_ERROR_UNSUPPORTED_PORT_DIRECTION);
}

TEST(ParseTest, UnpackedArray)
{
    const char *source_paths = "fixtures/unpacked_array/sample.sv";
    const char *top_module_name = "sample";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_TRUE(result.is_error);
    ASSERT_EQ(result.error, OOMBAK_PARSER_ERROR_UNSUPPORTED_SYMBOL_TYPE);
}

TEST(ParseTest, FileNotFound)
{
    const char *source_paths = "fixtures/invalid_folder/sample.sv";
    const char *top_module_name = "sample";

    auto result = oombak_parser_parse(source_paths, top_module_name);
    ASSERT_TRUE(result.is_error);
    ASSERT_EQ(result.error, OOMBAK_PARSER_ERROR_FILE_NOT_FOUND);
}

TEST(ParseTest, MultiThread)
{
    std::future<bool> is_errors_futures[ParseTest::NUM_OF_THREADS];
    for (int i = 0; i < ParseTest::NUM_OF_THREADS; i++)
    {
        is_errors_futures[i] = std::async(std::launch::async, ParseTest::is_parse_error);
    }
    for (int i = 0; i < ParseTest::NUM_OF_THREADS; i++)
    {
        ASSERT_FALSE(is_errors_futures[i].get());
    }
}

bool ParseTest::is_parse_error()
{
    const char *source_paths = "fixtures/sv_sample_1/sample.sv:fixtures/sv_sample_1/adder.sv:fixtures/"
                               "sv_sample_1/subtractor.sv";
    const char *top_module_name = "sample";

    auto ctx = oombak_parser_get_ctx();
    auto result = oombak_parser_parse_r(ctx, source_paths, top_module_name);
    auto is_error = result.is_error;
    oombak_parser_free_ctx(ctx);
    return is_error;
}
