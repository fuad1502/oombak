cmake_minimum_required(VERSION 3.15)
project(oombak_parser CXX)

SET(CMAKE_EXPORT_COMPILE_COMMANDS ON)
SET(CMAKE_POSITION_INDEPENDENT_CODE ON)

include(FetchContent)
FetchContent_Declare(
  slang
  GIT_REPOSITORY https://github.com/MikePopoloski/slang.git
  GIT_SHALLOW ON)
FetchContent_MakeAvailable(slang)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(BUILD_SHARED_LIBS "Build using shared libraries" OFF)

add_library(oombak_parser src/oombak_parser.cpp)
target_include_directories(oombak_parser PUBLIC include)
target_link_libraries(oombak_parser PUBLIC slang::slang)

set_target_properties(oombak_parser PROPERTIES PUBLIC_HEADER "include/oombak_parser.h")
install(TARGETS oombak_parser)

enable_testing()
add_subdirectory(tests)
