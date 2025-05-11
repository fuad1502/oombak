#include "oombak_parser.h"

#include "instance_tree_builder.hpp"
#include "slang/ast/Compilation.h"
#include "slang/ast/symbols/CompilationUnitSymbols.h"
#include "slang/syntax/SyntaxTree.h"
#include <cstdlib>
#include <exception>
#include <string_view>

using slang::ast::Compilation;
using slang::syntax::SyntaxTree;

static std::vector<std::string_view>
from_colon_separated_paths(const char *colon_separated_paths);

static void free_instance(Instance *instance);

class OombakParser {
public:
  OombakParser();
  ~OombakParser();
  Instance *get_instance_tree(const std::vector<std::string_view> &source_paths,
                              std::string_view top_module_name);

private:
  Instance root_instance;

  void add_syntax_trees(Compilation &compilation,
                        const std::vector<std::string_view> &source_paths);
  void check_compilation(Compilation &compilation);
};

static OombakParser *parser = new OombakParser();

OOMBAK_PARSER_EXPORT OombakCtx oombak_parser_get_ctx() {
  return new OombakParser();
}

OOMBAK_PARSER_EXPORT void oombak_parser_free_ctx(OombakCtx ctx) {
  auto parser = (OombakParser *)ctx;
  delete parser;
}

OOMBAK_PARSER_EXPORT Instance *
oombak_parser_parse(const char *source_paths, const char *top_module_name) {
  std::vector<std::string_view> source_paths_vec =
      from_colon_separated_paths(source_paths);
  try {
    return parser->get_instance_tree(source_paths_vec, top_module_name);
  } catch (...) {
    return NULL;
  }
}

OOMBAK_PARSER_EXPORT Instance *
oombak_parser_parse(OombakCtx ctx, const char *source_paths,
                    const char *top_module_name) {
  auto parser = (OombakParser *)ctx;
  std::vector<std::string_view> source_paths_vec =
      from_colon_separated_paths(source_paths);
  try {
    return parser->get_instance_tree(source_paths_vec, top_module_name);
  } catch (...) {
    return NULL;
  }
}

OombakParser::OombakParser() {}

OombakParser::~OombakParser() { free_instance(&root_instance); }

Instance *OombakParser::get_instance_tree(
    const std::vector<std::string_view> &source_paths,
    std::string_view top_module_name) {
  free_instance(&root_instance);
  InstanceTreeBuilder visitor(&root_instance);
  Compilation compilation;
  add_syntax_trees(compilation, source_paths);
  check_compilation(compilation);
  compilation.getRoot().visit(visitor);
  return &root_instance;
}

void OombakParser::add_syntax_trees(
    Compilation &compilation,
    const std::vector<std::string_view> &source_paths) {
  for (auto path : source_paths) {
    auto tree = SyntaxTree::fromFile(path).value();
    compilation.addSyntaxTree(tree);
  }
}

void OombakParser::check_compilation(Compilation &compilation) {
  if (!compilation.getAllDiagnostics().empty()) {
    throw new std::exception;
  }
}

std::vector<std::string_view>
from_colon_separated_paths(const char *colon_separated_paths) {
  std::vector<std::string_view> result;
  uint64_t input_length = strlen(colon_separated_paths);
  uint64_t last_idx = -1;
  for (int i = 0; i <= input_length; i++) {
    if (i == input_length || colon_separated_paths[i] == ':') {
      result.push_back(std::basic_string_view(
          &colon_separated_paths[last_idx + 1], &colon_separated_paths[i]));
      last_idx = i;
    }
  }
  return result;
}

void free_instance(Instance *instance) {
  delete instance->name;
  delete instance->module_name;
  for (int i = 0; i < instance->signals_len; i++) {
    delete instance->signals[i].name;
  }
  for (int i = 0; i < instance->child_instances_len; i++) {
    free_instance(instance->child_instances[i]);
  }
}
