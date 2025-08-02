include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)
include(GoogleTest)

add_executable(parser_test parse_test.cpp utils.cpp)
target_link_libraries(parser_test GTest::gtest_main oombak_parser)

gtest_discover_tests(parser_test WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
