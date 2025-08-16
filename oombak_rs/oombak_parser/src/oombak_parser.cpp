#include "oombak_parser.h"

#include <cstdio>
#include <cstdlib>
#include <string_view>

#include "instance_tree_builder.hpp"
#include "slang/ast/Compilation.h"
#include "slang/ast/symbols/CompilationUnitSymbols.h"
#include "slang/diagnostics/DiagnosticEngine.h"
#include "slang/diagnostics/NumericDiags.h"
#include "slang/diagnostics/TextDiagnosticClient.h"
#include "slang/syntax/SyntaxTree.h"

using slang::ast::Compilation;
using slang::syntax::SyntaxTree;

#define RETURN_ON_ERROR(f)                                                                                             \
    {                                                                                                                  \
        if (auto res = f)                                                                                              \
        {                                                                                                              \
            return f.value();                                                                                          \
        }                                                                                                              \
    }

namespace OombakParser
{

class OombakParser
{
  public:
    OombakParser();
    ~OombakParser();
    std::variant<oombak_parser_instance_t *, oombak_parser_error_t> get_instance_tree(
        const std::vector<std::string_view> &source_paths, std::string_view top_level_module_name);
    std::string get_last_diagnostics();

  private:
    oombak_parser_instance_t root_instance;
    std::string last_diagnostics;

    void set_last_diagnostics(Compilation &compilation);

    static std::optional<oombak_parser_error_t> add_syntax_trees(Compilation &compilation,
                                                                 const std::vector<std::string_view> &source_paths);
    static std::optional<oombak_parser_error_t> check_compilation(Compilation &compilation);
    static void free_instance(oombak_parser_instance_t *instance);
};

OombakParser::OombakParser()
{
    root_instance.parent_instance = NULL;
    root_instance.name = NULL;
    root_instance.module_name = NULL;
    root_instance.child_instances_len = 0;
    root_instance.child_instances = NULL;
    root_instance.signals_len = 0;
    root_instance.signals = NULL;
}

OombakParser::~OombakParser()
{
    free_instance(&root_instance);
}

std::variant<oombak_parser_instance_t *, oombak_parser_error_t> OombakParser::get_instance_tree(
    const std::vector<std::string_view> &source_paths, std::string_view top_level_module_name)
{
    free_instance(&root_instance);
    InstanceTreeBuilder visitor(&root_instance, top_level_module_name);
    Compilation compilation;
    RETURN_ON_ERROR(add_syntax_trees(compilation, source_paths));
    set_last_diagnostics(compilation);
    RETURN_ON_ERROR(check_compilation(compilation));
    compilation.getRoot().visit(visitor);
    if (visitor.has_error())
    {
        return visitor.get_error();
    }
    else if (!visitor.is_root_found())
    {
        return OOMBAK_PARSER_ERROR_TOP_LEVEL_MODULE_NOT_FOUND;
    }
    return &root_instance;
}

std::optional<oombak_parser_error_t> OombakParser::add_syntax_trees(Compilation &compilation,
                                                                    const std::vector<std::string_view> &source_paths)
{
    try
    {
        for (auto path : source_paths)
        {
            auto tree = SyntaxTree::fromFile(path).value();
            compilation.addSyntaxTree(tree);
        }
        return std::nullopt;
    }
    catch (...)
    {
        return OOMBAK_PARSER_ERROR_FILE_NOT_FOUND;
    }
}

std::optional<oombak_parser_error_t> OombakParser::check_compilation(Compilation &compilation)
{
    if (!compilation.getAllDiagnostics().empty())
    {
        return OOMBAK_PARSER_ERROR_COMPILE_ERROR;
    }
    return std::nullopt;
}

void OombakParser::set_last_diagnostics(Compilation &compilation)
{
    auto source_manager = compilation.getSourceManager();
    auto diagnostics = compilation.getAllDiagnostics();
    last_diagnostics = slang::DiagnosticEngine::reportAll(*source_manager, diagnostics);
}

std::string OombakParser::get_last_diagnostics()
{
    return last_diagnostics;
}

std::vector<std::string_view> from_colon_separated_paths(const char *colon_separated_paths)
{
    std::vector<std::string_view> result;
    uint64_t input_length = strlen(colon_separated_paths);
    uint64_t last_idx = -1;
    for (int i = 0; i <= input_length; i++)
    {
        if (i == input_length || colon_separated_paths[i] == ':')
        {
            result.push_back(std::basic_string_view(&colon_separated_paths[last_idx + 1], &colon_separated_paths[i]));
            last_idx = i;
        }
    }
    return result;
}

void OombakParser::free_instance(oombak_parser_instance_t *instance)
{
    free((void *)instance->name);
    free((void *)instance->module_name);
    for (int i = 0; i < instance->signals_len; i++)
    {
        free((void *)instance->signals[i].name);
    }
    free(instance->signals);
    for (int i = 0; i < instance->child_instances_len; i++)
    {
        free_instance(instance->child_instances[i]);
    }
}

} // namespace OombakParser

static OombakParser::OombakParser *parser = new OombakParser::OombakParser();

static oombak_parser_result_t instance_or_error_variant_to_result(
    std::variant<oombak_parser_instance_t *, oombak_parser_error_t> instance_or_error);
static std::vector<std::string_view> from_colon_separated_paths(const char *colon_separated_paths);

oombak_parser_ctx_t oombak_parser_get_ctx()
{
    return new OombakParser::OombakParser();
}

void oombak_parser_free_ctx(oombak_parser_ctx_t ctx)
{
    auto parser = (OombakParser::OombakParser *)ctx;
    delete parser;
}

oombak_parser_result_t oombak_parser_parse(const char *source_paths, const char *top_level_module_name)
{
    std::vector<std::string_view> source_paths_vec = OombakParser::from_colon_separated_paths(source_paths);
    auto instance_or_error = parser->get_instance_tree(source_paths_vec, top_level_module_name);
    return instance_or_error_variant_to_result(instance_or_error);
}

oombak_parser_result_t oombak_parser_parse_r(oombak_parser_ctx_t ctx, const char *source_paths,
                                             const char *top_level_module_name)
{
    auto parser = (OombakParser::OombakParser *)ctx;
    std::vector<std::string_view> source_paths_vec = OombakParser::from_colon_separated_paths(source_paths);
    auto instance_or_error = parser->get_instance_tree(source_paths_vec, top_level_module_name);
    return instance_or_error_variant_to_result(instance_or_error);
}

const char *oombak_parser_get_last_diagnostics()
{
    return strdup(parser->get_last_diagnostics().c_str());
}

const char *oombak_parser_get_last_diagnostics_r(oombak_parser_ctx_t ctx)
{
    auto parser = (OombakParser::OombakParser *)ctx;
    return strdup(parser->get_last_diagnostics().c_str());
}

oombak_parser_result_t instance_or_error_variant_to_result(
    std::variant<oombak_parser_instance_t *, oombak_parser_error_t> instance_or_error)
{
    oombak_parser_result_t result;
    if (std::holds_alternative<oombak_parser_instance_t *>(instance_or_error))
    {
        result.is_error = false;
        result.instance = std::get<oombak_parser_instance_t *>(instance_or_error);
    }
    else
    {
        result.is_error = true;
        result.error = std::get<oombak_parser_error_t>(instance_or_error);
    }
    return result;
}
