include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/52eb8108c5bdec04579160ae17225d66034bd723.zip
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)
include(GoogleTest)

add_executable(parser_test parse_test.cpp utils.cpp)
target_link_libraries(parser_test GTest::gtest_main oombak_parser)

gtest_discover_tests(parser_test WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
