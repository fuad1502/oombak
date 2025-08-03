#ifndef OOMBAK_PARSER_H
#define OOMBAK_PARSER_H

#include <cstdint>

#ifdef _WIN32
#define OOMBAK_PARSER_EXPORT __declspec(dllexport)
#else
#define OOMBAK_PARSER_EXPORT
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef enum oombak_parser_signal_type {
  OOMBAK_PARSER_PACKED_ARR_PORT_IN,
  OOMBAK_PARSER_PACKED_ARR_PORT_OUT,
  OOMBAK_PARSER_PACKED_ARR_VAR_NET,
} oombak_parser_signal_type_t;

typedef struct oombak_parser_signal {
  const char *name;
  oombak_parser_signal_type_t type;
  uint64_t width;
} oombak_parser_signal_t;

typedef struct oombak_parser_instance {
  const char *name;
  const char *module_name;
  struct oombak_parser_instance *parent_instance;
  struct oombak_parser_instance **child_instances;
  uint64_t child_instances_len;
  oombak_parser_signal_t *signals;
  uint64_t signals_len;
} oombak_parser_instance_t;

typedef enum oombak_parser_error {
  OOMBAK_PARSER_ERROR_NONE,
  OOMBAK_PARSER_ERROR_FILE_NOT_FOUND,
  OOMBAK_PARSER_ERROR_TOP_MODULE_NOT_FOUND,
  OOMBAK_PARSER_ERROR_COMPILE_ERROR,
  OOMBAK_PARSER_ERROR_UNSUPPORTED_SYMBOL_TYPE,
  OOMBAK_PARSER_ERROR_UNSUPPORTED_PORT_DIRECTION,
} oombak_parser_error_t;

typedef struct oombak_parser_result {
  uint8_t is_error;
  union {
    oombak_parser_instance_t *instance;
    oombak_parser_error_t error;
  };
} oombak_parser_result_t;

typedef void *oombak_parser_ctx_t;

OOMBAK_PARSER_EXPORT oombak_parser_ctx_t oombak_parser_get_ctx();

OOMBAK_PARSER_EXPORT oombak_parser_result_t
oombak_parser_parse(const char *source_paths, const char *top_module_name);

OOMBAK_PARSER_EXPORT oombak_parser_result_t
oombak_parser_parse_r(oombak_parser_ctx_t ctx, const char *source_paths,
                      const char *top_module_name);

OOMBAK_PARSER_EXPORT void oombak_parser_free_ctx(oombak_parser_ctx_t ctx);

#ifdef __cplusplus
}
#endif

#endif // OOMBAK_PARSER_H
