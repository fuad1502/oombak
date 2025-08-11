#pragma once

#include "oombak_parser.h"
#include "slang/ast/ASTVisitor.h"
#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <vector>

using slang::ast::ASTVisitor;
using slang::ast::InstanceSymbol;
using slang::ast::NetSymbol;
using slang::ast::PortSymbol;
using slang::ast::Scope;
using slang::ast::VariableSymbol;
using std::string;
using std::vector;

namespace OombakParser
{

class Exception : public std::exception
{
  public:
    Exception(oombak_parser_error_t error)
    {
        this->error = error;
    }
    oombak_parser_error_t get_error_t()
    {
        return error;
    }

  private:
    oombak_parser_error_t error;
};

class InstanceTreeBuilder : public ASTVisitor<InstanceTreeBuilder, true, true>
{
  public:
    InstanceTreeBuilder(oombak_parser_instance_t *root_instance, std::string_view top_level_module_name)
    {
        this->root_instance = root_instance;
        this->top_level_module_name = top_level_module_name;
        root_instance->parent_instance = NULL;
        root_instance->name = NULL;
        root_instance->module_name = NULL;
        root_instance->child_instances_len = 0;
        root_instance->child_instances = NULL;
        root_instance->signals_len = 0;
        root_instance->signals = NULL;
        error = OOMBAK_PARSER_ERROR_NONE;
    }

    bool is_root_found()
    {
        return root_instance->name != NULL;
    }

    bool has_error()
    {
        return error != OOMBAK_PARSER_ERROR_NONE;
    }

    oombak_parser_error_t get_error()
    {
        return error;
    }

    void handle(const InstanceSymbol &s)
    {
        if (has_error() || is_root_found())
        {
            return;
        }
        else if (module_name(s) == top_level_module_name)
        {
            try
            {
                visitInstance(s, root_instance);
            }
            catch (Exception &e)
            {
                error = e.get_error_t();
            }
        }
        else
        {
            for (auto it = s.body.membersOfType<InstanceSymbol>().begin();
                 it != s.body.membersOfType<InstanceSymbol>().end(); it.increment())
            {
                visitDefault(s);
            }
        }
    }

  private:
    oombak_parser_instance_t *root_instance;
    std::string_view top_level_module_name;
    oombak_parser_error_t error;

    void visitInstance(const InstanceSymbol &symbol, oombak_parser_instance_t *instance)
    {
        set_name(instance, symbol);
        auto signals = get_signals(symbol);
        set_signals(instance, signals);
        auto child_instances = visit_and_get_child_instances(symbol);
        set_child_instances(instance, child_instances);
    }

    void set_name(oombak_parser_instance_t *instance, const InstanceSymbol &symbol)
    {
        if (is_root_found())
        {
            instance->name = strdup(string(symbol.name).c_str());
        }
        else
        {
            instance->name = strdup(string(symbol.body.name).c_str());
        }
        instance->module_name = strdup(string(symbol.body.name).c_str());
    }

    vector<oombak_parser_signal_t> get_signals(const InstanceSymbol &symbol)
    {
        vector<oombak_parser_signal_t> signals;
        append_with_port_signals(signals, symbol);
        append_with_net_signals(signals, symbol);
        append_with_var_signals(signals, symbol);
        return signals;
    }

    void set_signals(oombak_parser_instance_t *instance, const vector<oombak_parser_signal_t> &signals)
    {
        instance->signals_len = signals.size();
        instance->signals = (oombak_parser_signal_t *)malloc(signals.size() * sizeof(oombak_parser_signal_t));
        for (int i = 0; i < signals.size(); i++)
        {
            instance->signals[i] = signals[i];
        }
    }

    vector<oombak_parser_instance_t *> visit_and_get_child_instances(const InstanceSymbol &symbol)
    {
        vector<oombak_parser_instance_t *> child_instances;
        for (auto it = symbol.body.membersOfType<InstanceSymbol>().begin();
             it != symbol.body.membersOfType<InstanceSymbol>().end(); it.increment())
        {
            oombak_parser_instance_t *child_instance =
                (oombak_parser_instance_t *)malloc(sizeof(oombak_parser_instance_t));
            child_instances.push_back(child_instance);
            visitInstance(*it, child_instance);
        }
        return child_instances;
    }

    void set_child_instances(oombak_parser_instance_t *instance,
                             const vector<oombak_parser_instance_t *> &child_instances)
    {
        instance->child_instances_len = child_instances.size();
        instance->child_instances =
            (oombak_parser_instance_t **)malloc(child_instances.size() * sizeof(oombak_parser_instance_t *));
        for (int i = 0; i < child_instances.size(); i++)
        {
            child_instances[i]->parent_instance = instance;
            instance->child_instances[i] = child_instances[i];
        }
    }

    void append_with_port_signals(vector<oombak_parser_signal_t> &signals, const InstanceSymbol &symbol)
    {
        append_with_signals_of_type<PortSymbol>(signals, symbol);
    }

    void append_with_net_signals(vector<oombak_parser_signal_t> &signals, const InstanceSymbol &symbol)
    {
        append_with_signals_of_type<NetSymbol>(signals, symbol);
    }

    void append_with_var_signals(vector<oombak_parser_signal_t> &signals, const InstanceSymbol &symbol)
    {
        append_with_signals_of_type<VariableSymbol>(signals, symbol);
    }

    template <typename T>
    void append_with_signals_of_type(vector<oombak_parser_signal_t> &signals, const InstanceSymbol &symbol)
    {
        for (auto it = symbol.body.membersOfType<T>().begin(); it != symbol.body.membersOfType<T>().end();
             it.increment())
        {
            throw_if_unsupported_symbol_type<T>(it);
            oombak_parser_signal_t sig;
            sig.name = strdup(string(it->name).c_str());
            if (is_port_with_name_inside(sig.name, signals))
            {
                continue;
            }
            sig.width = get_signal_width<T>(it);
            if constexpr (std::is_same_v<PortSymbol, T>)
                sig.type = get_port_type(it);
            else
                sig.type = OOMBAK_PARSER_PACKED_ARR_VAR_NET;
            signals.push_back(sig);
        }
    }

    template <typename T> void throw_if_unsupported_symbol_type(Scope::specific_symbol_iterator<T> symbol)
    {
        if (!(symbol->getType().isPackedArray() || symbol->getType().isScalar()))
        {
            throw Exception(OOMBAK_PARSER_ERROR_UNSUPPORTED_SYMBOL_TYPE);
        }
    }

    oombak_parser_signal_type_t get_port_type(Scope::specific_symbol_iterator<PortSymbol> symbol)
    {
        switch (symbol->direction)
        {
        case slang::ast::ArgumentDirection::In:
            return OOMBAK_PARSER_PACKED_ARR_PORT_IN;
        case slang::ast::ArgumentDirection::Out:
            return OOMBAK_PARSER_PACKED_ARR_PORT_OUT;
        case slang::ast::ArgumentDirection::InOut:
        case slang::ast::ArgumentDirection::Ref:
            break;
        }
        throw Exception(OOMBAK_PARSER_ERROR_UNSUPPORTED_PORT_DIRECTION);
    }

    template <typename T> uint64_t get_signal_width(Scope::specific_symbol_iterator<T> symbol)
    {
        return symbol->getType().getBitWidth();
    }

    static bool is_port_with_name_inside(const char *name, const vector<oombak_parser_signal_t> &signals)
    {
        return std::find_if(signals.begin(), signals.end(), port_with_name(name)) != signals.end();
    }

    static bool is_port(const oombak_parser_signal_t &s)
    {
        return (s.type == OOMBAK_PARSER_PACKED_ARR_PORT_IN || s.type == OOMBAK_PARSER_PACKED_ARR_PORT_OUT);
    }

    static std::function<bool(oombak_parser_signal_t)> port_with_name(const char *name)
    {
        return [name](oombak_parser_signal_t s) {
            if (is_port(s) && strcmp(s.name, name) == 0)
                return true;
            else
                return false;
        };
    }

    const std::string_view &module_name(const InstanceSymbol &s)
    {
        return s.body.name;
    }
};

} // namespace OombakParser
