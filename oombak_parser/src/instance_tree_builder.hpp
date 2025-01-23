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

class InstanceTreeBuilder : public ASTVisitor<InstanceTreeBuilder, true, true> {
public:
  InstanceTreeBuilder(Instance *root_instance) {
    this->root_instance = root_instance;
  }

  void handle(const InstanceSymbol &s) {
    root_instance->parent_instance = NULL;
    visitInstance(s, root_instance);
  }

private:
  Instance *root_instance;

  void visitInstance(const InstanceSymbol &symbol, Instance *instance) {
    set_name(instance, symbol);
    auto signals = get_signals(symbol);
    set_signals(instance, signals);
    auto child_instances = visit_and_get_child_instances(symbol);
    set_child_instances(instance, child_instances);
  }

  void set_name(Instance *instance, const InstanceSymbol &symbol) {
    instance->name = strdup(string(symbol.name).c_str());
    instance->module_name = strdup(string(symbol.body.name).c_str());
  }

  vector<Signal> get_signals(const InstanceSymbol &symbol) {
    vector<Signal> signals;
    append_with_port_signals(signals, symbol);
    append_with_net_signals(signals, symbol);
    append_with_var_signals(signals, symbol);
    return signals;
  }

  void set_signals(Instance *instance, const vector<Signal> &signals) {
    instance->signals_len = signals.size();
    instance->signals = (Signal *)malloc(signals.size() * sizeof(Signal));
    for (int i = 0; i < signals.size(); i++) {
      instance->signals[i] = signals[i];
    }
  }

  vector<Instance *>
  visit_and_get_child_instances(const InstanceSymbol &symbol) {
    vector<Instance *> child_instances;
    for (auto it = symbol.body.membersOfType<InstanceSymbol>().begin();
         it != symbol.body.membersOfType<InstanceSymbol>().end();
         it.increment()) {
      Instance *child_instance = (Instance *)malloc(sizeof(Instance));
      child_instances.push_back(child_instance);
      visitInstance(*it, child_instance);
    }
    return child_instances;
  }

  void set_child_instances(Instance *instance,
                           const vector<Instance *> &child_instances) {
    instance->child_instances_len = child_instances.size();
    instance->child_instances =
        (Instance **)malloc(child_instances.size() * sizeof(Instance *));
    for (int i = 0; i < child_instances.size(); i++) {
      child_instances[i]->parent_instance = instance;
      instance->child_instances[i] = child_instances[i];
    }
  }

  void append_with_port_signals(vector<Signal> &signals,
                                const InstanceSymbol &symbol) {
    append_with_signals_of_type<PortSymbol>(signals, symbol);
  }

  void append_with_net_signals(vector<Signal> &signals,
                               const InstanceSymbol &symbol) {
    append_with_signals_of_type<NetSymbol>(signals, symbol);
  }

  void append_with_var_signals(vector<Signal> &signals,
                               const InstanceSymbol &symbol) {
    append_with_signals_of_type<VariableSymbol>(signals, symbol);
  }

  template <typename T>
  void append_with_signals_of_type(vector<Signal> &signals,
                                   const InstanceSymbol &symbol) {
    for (auto it = symbol.body.membersOfType<T>().begin();
         it != symbol.body.membersOfType<T>().end(); it.increment()) {
      Signal sig;
      sig.name = strdup(string(it->name).c_str());
      if (is_port_with_name_inside(sig.name, signals)) {
        continue;
      }
      sig.width = get_signal_width<T>(it);
      if constexpr (std::is_same_v<PortSymbol, T>)
        sig.type = get_port_type(it);
      else
        sig.type = UnpackedArrVarNet;
      signals.push_back(sig);
    }
  }

  SignalType get_port_type(Scope::specific_symbol_iterator<PortSymbol> symbol) {
    switch (symbol->direction) {
    case slang::ast::ArgumentDirection::In:
      return UnpackedArrPortIn;
    case slang::ast::ArgumentDirection::Out:
      return UnpackedArrPortOut;
    case slang::ast::ArgumentDirection::InOut:
    case slang::ast::ArgumentDirection::Ref:
      break;
    }
    throw new std::exception();
  }

  template <typename T>
  uint64_t get_signal_width(Scope::specific_symbol_iterator<T> symbol) {
    return symbol->getType().getBitWidth();
  }

  static bool is_port_with_name_inside(const char *name,
                                       const vector<Signal> &signals) {
    return std::find_if(signals.begin(), signals.end(), port_with_name(name)) !=
           signals.end();
  }

  static bool is_port(const Signal &s) {
    return (s.type == UnpackedArrPortOut || s.type == UnpackedArrPortIn);
  }

  static std::function<bool(Signal)> port_with_name(const char *name) {
    return [name](Signal s) {
      if (is_port(s) && strcmp(s.name, name) == 0)
        return true;
      else
        return false;
    };
  }
};
