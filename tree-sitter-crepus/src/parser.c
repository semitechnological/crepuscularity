#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 34
#define LARGE_STATE_COUNT 2
#define SYMBOL_COUNT 38
#define ALIAS_COUNT 0
#define TOKEN_COUNT 22
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 5
#define MAX_ALIAS_SEQUENCE_LENGTH 3
#define PRODUCTION_ID_COUNT 7

enum ts_symbol_identifiers {
  anon_sym_LF = 1,
  sym_comment = 2,
  sym_frontmatter_marker = 3,
  anon_sym_DASH_DASH_DASH = 4,
  aux_sym_fragment_section_token1 = 5,
  anon_sym_LT = 6,
  aux_sym_jsx_fragment_token1 = 7,
  anon_sym_GT = 8,
  anon_sym_LT_SLASH = 9,
  anon_sym_SLASH_GT = 10,
  anon_sym_DQUOTE = 11,
  aux_sym_quoted_token1 = 12,
  aux_sym_quoted_token2 = 13,
  sym_element_tag = 14,
  anon_sym_LBRACE = 15,
  anon_sym_RBRACE = 16,
  sym_braced_body = 17,
  sym_hash_id = 18,
  sym_attr_name_eq = 19,
  aux_sym_tailwind_pair_token1 = 20,
  aux_sym_plain_class_token1 = 21,
  sym_template = 22,
  sym__eol = 23,
  sym_logical_line = 24,
  sym_fragment_section = 25,
  sym_jsx_fragment = 26,
  sym_quoted = 27,
  sym_element_line = 28,
  sym_class_segment = 29,
  sym_braced_expression = 30,
  sym_attr_binding_braced = 31,
  sym_attr_name_only = 32,
  sym_tailwind_pair = 33,
  sym_plain_class = 34,
  aux_sym_template_repeat1 = 35,
  aux_sym_quoted_repeat1 = 36,
  aux_sym_element_line_repeat1 = 37,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_LF] = "\n",
  [sym_comment] = "comment",
  [sym_frontmatter_marker] = "frontmatter_marker",
  [anon_sym_DASH_DASH_DASH] = "---",
  [aux_sym_fragment_section_token1] = "fragment_section_token1",
  [anon_sym_LT] = "<",
  [aux_sym_jsx_fragment_token1] = "jsx_fragment_token1",
  [anon_sym_GT] = ">",
  [anon_sym_LT_SLASH] = "</",
  [anon_sym_SLASH_GT] = "/>",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym_quoted_token1] = "quoted_token1",
  [aux_sym_quoted_token2] = "quoted_token2",
  [sym_element_tag] = "element_tag",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [sym_braced_body] = "braced_body",
  [sym_hash_id] = "hash_id",
  [sym_attr_name_eq] = "attr_name_eq",
  [aux_sym_tailwind_pair_token1] = "tailwind_pair_token1",
  [aux_sym_plain_class_token1] = "plain_class_token1",
  [sym_template] = "template",
  [sym__eol] = "_eol",
  [sym_logical_line] = "logical_line",
  [sym_fragment_section] = "fragment_section",
  [sym_jsx_fragment] = "jsx_fragment",
  [sym_quoted] = "quoted",
  [sym_element_line] = "element_line",
  [sym_class_segment] = "class_segment",
  [sym_braced_expression] = "braced_expression",
  [sym_attr_binding_braced] = "attr_binding_braced",
  [sym_attr_name_only] = "attr_name_only",
  [sym_tailwind_pair] = "tailwind_pair",
  [sym_plain_class] = "plain_class",
  [aux_sym_template_repeat1] = "template_repeat1",
  [aux_sym_quoted_repeat1] = "quoted_repeat1",
  [aux_sym_element_line_repeat1] = "element_line_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_LF] = anon_sym_LF,
  [sym_comment] = sym_comment,
  [sym_frontmatter_marker] = sym_frontmatter_marker,
  [anon_sym_DASH_DASH_DASH] = anon_sym_DASH_DASH_DASH,
  [aux_sym_fragment_section_token1] = aux_sym_fragment_section_token1,
  [anon_sym_LT] = anon_sym_LT,
  [aux_sym_jsx_fragment_token1] = aux_sym_jsx_fragment_token1,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_LT_SLASH] = anon_sym_LT_SLASH,
  [anon_sym_SLASH_GT] = anon_sym_SLASH_GT,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym_quoted_token1] = aux_sym_quoted_token1,
  [aux_sym_quoted_token2] = aux_sym_quoted_token2,
  [sym_element_tag] = sym_element_tag,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [sym_braced_body] = sym_braced_body,
  [sym_hash_id] = sym_hash_id,
  [sym_attr_name_eq] = sym_attr_name_eq,
  [aux_sym_tailwind_pair_token1] = aux_sym_tailwind_pair_token1,
  [aux_sym_plain_class_token1] = aux_sym_plain_class_token1,
  [sym_template] = sym_template,
  [sym__eol] = sym__eol,
  [sym_logical_line] = sym_logical_line,
  [sym_fragment_section] = sym_fragment_section,
  [sym_jsx_fragment] = sym_jsx_fragment,
  [sym_quoted] = sym_quoted,
  [sym_element_line] = sym_element_line,
  [sym_class_segment] = sym_class_segment,
  [sym_braced_expression] = sym_braced_expression,
  [sym_attr_binding_braced] = sym_attr_binding_braced,
  [sym_attr_name_only] = sym_attr_name_only,
  [sym_tailwind_pair] = sym_tailwind_pair,
  [sym_plain_class] = sym_plain_class,
  [aux_sym_template_repeat1] = aux_sym_template_repeat1,
  [aux_sym_quoted_repeat1] = aux_sym_quoted_repeat1,
  [aux_sym_element_line_repeat1] = aux_sym_element_line_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_LF] = {
    .visible = true,
    .named = false,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [sym_frontmatter_marker] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_DASH_DASH_DASH] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_fragment_section_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_jsx_fragment_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_quoted_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_quoted_token2] = {
    .visible = false,
    .named = false,
  },
  [sym_element_tag] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [sym_braced_body] = {
    .visible = true,
    .named = true,
  },
  [sym_hash_id] = {
    .visible = true,
    .named = true,
  },
  [sym_attr_name_eq] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_tailwind_pair_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_plain_class_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_template] = {
    .visible = true,
    .named = true,
  },
  [sym__eol] = {
    .visible = false,
    .named = true,
  },
  [sym_logical_line] = {
    .visible = true,
    .named = true,
  },
  [sym_fragment_section] = {
    .visible = true,
    .named = true,
  },
  [sym_jsx_fragment] = {
    .visible = true,
    .named = true,
  },
  [sym_quoted] = {
    .visible = true,
    .named = true,
  },
  [sym_element_line] = {
    .visible = true,
    .named = true,
  },
  [sym_class_segment] = {
    .visible = true,
    .named = true,
  },
  [sym_braced_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_attr_binding_braced] = {
    .visible = true,
    .named = true,
  },
  [sym_attr_name_only] = {
    .visible = true,
    .named = true,
  },
  [sym_tailwind_pair] = {
    .visible = true,
    .named = true,
  },
  [sym_plain_class] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_template_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_quoted_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_element_line_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_attr = 1,
  field_body = 2,
  field_class = 3,
  field_tag = 4,
  field_value = 5,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_attr] = "attr",
  [field_body] = "body",
  [field_class] = "class",
  [field_tag] = "tag",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
  [4] = {.index = 4, .length = 2},
  [5] = {.index = 6, .length = 2},
  [6] = {.index = 8, .length = 1},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_tag, 0},
  [1] =
    {field_class, 0},
  [2] =
    {field_class, 1, .inherited = true},
    {field_tag, 0},
  [4] =
    {field_attr, 0},
    {field_value, 1},
  [6] =
    {field_class, 0, .inherited = true},
    {field_class, 1, .inherited = true},
  [8] =
    {field_body, 1},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(9);
      ADVANCE_MAP(
        '\n', 10,
        '"', 26,
        '#', 11,
        '+', 43,
        '-', 45,
        '/', 50,
        '<', 20,
        '>', 23,
        '{', 35,
        '}', 36,
      );
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(0);
      if (('@' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(49);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(52);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(26);
      if (lookahead == '\\') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(29);
      if (lookahead != 0) ADVANCE(28);
      END_STATE();
    case 2:
      if (lookahead == '/') ADVANCE(3);
      if (lookahead == '>') ADVANCE(23);
      if (lookahead == '}') ADVANCE(36);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') SKIP(2);
      END_STATE();
    case 3:
      if (lookahead == '>') ADVANCE(25);
      END_STATE();
    case 4:
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(21);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead) &&
          lookahead != '>') ADVANCE(22);
      END_STATE();
    case 5:
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(18);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead)) ADVANCE(19);
      END_STATE();
    case 6:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(27);
      END_STATE();
    case 7:
      if (eof) ADVANCE(9);
      if (lookahead == '\n') ADVANCE(10);
      if (lookahead == '"') ADVANCE(26);
      if (lookahead == '#') ADVANCE(13);
      if (lookahead == '+') ADVANCE(31);
      if (lookahead == '-') ADVANCE(33);
      if (lookahead == '<') ADVANCE(20);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(7);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(34);
      END_STATE();
    case 8:
      if (eof) ADVANCE(9);
      if (lookahead == '\n') ADVANCE(10);
      if (lookahead == '#') ADVANCE(51);
      if (lookahead == '@') ADVANCE(49);
      if (lookahead == '{') ADVANCE(35);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(8);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          lookahead == '[' ||
          lookahead == ']' ||
          lookahead == '_') ADVANCE(48);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(46);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 9:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 10:
      ACCEPT_TOKEN(anon_sym_LF);
      END_STATE();
    case 11:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ' ||
          lookahead == '"' ||
          lookahead == '<') ADVANCE(13);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(11);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(12);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ' ||
          lookahead == '"' ||
          lookahead == '<') ADVANCE(13);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(12);
      END_STATE();
    case 13:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(13);
      END_STATE();
    case 14:
      ACCEPT_TOKEN(sym_frontmatter_marker);
      END_STATE();
    case 15:
      ACCEPT_TOKEN(sym_frontmatter_marker);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(anon_sym_DASH_DASH_DASH);
      END_STATE();
    case 17:
      ACCEPT_TOKEN(anon_sym_DASH_DASH_DASH);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(18);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead)) ADVANCE(19);
      END_STATE();
    case 19:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(19);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '/') ADVANCE(24);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(21);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead) &&
          lookahead != '>') ADVANCE(22);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '>') ADVANCE(22);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(anon_sym_GT);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(anon_sym_LT_SLASH);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(anon_sym_SLASH_GT);
      END_STATE();
    case 26:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(aux_sym_quoted_token1);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(aux_sym_quoted_token2);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(aux_sym_quoted_token2);
      if (lookahead == '\\') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(29);
      if (lookahead != 0 &&
          lookahead != '"') ADVANCE(28);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '+') ADVANCE(15);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '+') ADVANCE(30);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '-') ADVANCE(17);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '-') ADVANCE(32);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(sym_braced_body);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(37);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead) &&
          lookahead != '}') ADVANCE(38);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(sym_braced_body);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '}') ADVANCE(38);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(sym_hash_id);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(39);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(sym_attr_name_eq);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(aux_sym_tailwind_pair_token1);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(41);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '+') ADVANCE(14);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '+') ADVANCE(42);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '-') ADVANCE(16);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '-') ADVANCE(44);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '.') ADVANCE(49);
      if (lookahead == ':') ADVANCE(47);
      if (lookahead == '=') ADVANCE(40);
      if (lookahead == '[' ||
          lookahead == ']') ADVANCE(48);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(46);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<' &&
          lookahead != '=') ADVANCE(52);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '/') ADVANCE(52);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(41);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == ':') ADVANCE(47);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= '[') ||
          lookahead == ']' ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(48);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '=') ADVANCE(40);
      if (lookahead == '-' ||
          lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(49);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<' &&
          lookahead != '=') ADVANCE(52);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '>') ADVANCE(25);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(39);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(aux_sym_plain_class_token1);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(52);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 7},
  [2] = {.lex_state = 7},
  [3] = {.lex_state = 7},
  [4] = {.lex_state = 8},
  [5] = {.lex_state = 8},
  [6] = {.lex_state = 8},
  [7] = {.lex_state = 7},
  [8] = {.lex_state = 8},
  [9] = {.lex_state = 8},
  [10] = {.lex_state = 8},
  [11] = {.lex_state = 8},
  [12] = {.lex_state = 8},
  [13] = {.lex_state = 8},
  [14] = {.lex_state = 8},
  [15] = {.lex_state = 1},
  [16] = {.lex_state = 1},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 0},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 2},
  [24] = {.lex_state = 0},
  [25] = {.lex_state = 0},
  [26] = {.lex_state = 0},
  [27] = {.lex_state = 4},
  [28] = {.lex_state = 4},
  [29] = {.lex_state = 0},
  [30] = {.lex_state = 5},
  [31] = {.lex_state = 37},
  [32] = {.lex_state = 2},
  [33] = {.lex_state = 2},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_LF] = ACTIONS(1),
    [sym_comment] = ACTIONS(1),
    [sym_frontmatter_marker] = ACTIONS(1),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_LT_SLASH] = ACTIONS(1),
    [anon_sym_SLASH_GT] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [sym_hash_id] = ACTIONS(1),
    [sym_attr_name_eq] = ACTIONS(1),
    [aux_sym_plain_class_token1] = ACTIONS(1),
  },
  [1] = {
    [sym_template] = STATE(29),
    [sym__eol] = STATE(3),
    [sym_logical_line] = STATE(18),
    [sym_fragment_section] = STATE(20),
    [sym_jsx_fragment] = STATE(20),
    [sym_quoted] = STATE(20),
    [sym_element_line] = STATE(20),
    [aux_sym_template_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_LF] = ACTIONS(5),
    [sym_comment] = ACTIONS(7),
    [sym_frontmatter_marker] = ACTIONS(9),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_SLASH] = ACTIONS(15),
    [anon_sym_DQUOTE] = ACTIONS(17),
    [sym_element_tag] = ACTIONS(19),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 12,
    ACTIONS(21), 1,
      ts_builtin_sym_end,
    ACTIONS(23), 1,
      anon_sym_LF,
    ACTIONS(26), 1,
      sym_comment,
    ACTIONS(29), 1,
      sym_frontmatter_marker,
    ACTIONS(32), 1,
      anon_sym_DASH_DASH_DASH,
    ACTIONS(35), 1,
      anon_sym_LT,
    ACTIONS(38), 1,
      anon_sym_LT_SLASH,
    ACTIONS(41), 1,
      anon_sym_DQUOTE,
    ACTIONS(44), 1,
      sym_element_tag,
    STATE(26), 1,
      sym_logical_line,
    STATE(2), 2,
      sym__eol,
      aux_sym_template_repeat1,
    STATE(20), 4,
      sym_fragment_section,
      sym_jsx_fragment,
      sym_quoted,
      sym_element_line,
  [41] = 12,
    ACTIONS(7), 1,
      sym_comment,
    ACTIONS(9), 1,
      sym_frontmatter_marker,
    ACTIONS(11), 1,
      anon_sym_DASH_DASH_DASH,
    ACTIONS(13), 1,
      anon_sym_LT,
    ACTIONS(15), 1,
      anon_sym_LT_SLASH,
    ACTIONS(17), 1,
      anon_sym_DQUOTE,
    ACTIONS(19), 1,
      sym_element_tag,
    ACTIONS(47), 1,
      ts_builtin_sym_end,
    ACTIONS(49), 1,
      anon_sym_LF,
    STATE(19), 1,
      sym_logical_line,
    STATE(2), 2,
      sym__eol,
      aux_sym_template_repeat1,
    STATE(20), 4,
      sym_fragment_section,
      sym_jsx_fragment,
      sym_quoted,
      sym_element_line,
  [82] = 9,
    ACTIONS(53), 1,
      anon_sym_LBRACE,
    ACTIONS(55), 1,
      sym_hash_id,
    ACTIONS(57), 1,
      sym_attr_name_eq,
    ACTIONS(59), 1,
      aux_sym_tailwind_pair_token1,
    ACTIONS(61), 1,
      aux_sym_plain_class_token1,
    STATE(6), 1,
      aux_sym_element_line_repeat1,
    STATE(10), 1,
      sym_class_segment,
    ACTIONS(51), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    STATE(9), 5,
      sym_braced_expression,
      sym_attr_binding_braced,
      sym_attr_name_only,
      sym_tailwind_pair,
      sym_plain_class,
  [115] = 9,
    ACTIONS(53), 1,
      anon_sym_LBRACE,
    ACTIONS(55), 1,
      sym_hash_id,
    ACTIONS(57), 1,
      sym_attr_name_eq,
    ACTIONS(59), 1,
      aux_sym_tailwind_pair_token1,
    ACTIONS(61), 1,
      aux_sym_plain_class_token1,
    STATE(4), 1,
      aux_sym_element_line_repeat1,
    STATE(10), 1,
      sym_class_segment,
    ACTIONS(63), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    STATE(9), 5,
      sym_braced_expression,
      sym_attr_binding_braced,
      sym_attr_name_only,
      sym_tailwind_pair,
      sym_plain_class,
  [148] = 9,
    ACTIONS(67), 1,
      anon_sym_LBRACE,
    ACTIONS(70), 1,
      sym_hash_id,
    ACTIONS(73), 1,
      sym_attr_name_eq,
    ACTIONS(76), 1,
      aux_sym_tailwind_pair_token1,
    ACTIONS(79), 1,
      aux_sym_plain_class_token1,
    STATE(6), 1,
      aux_sym_element_line_repeat1,
    STATE(10), 1,
      sym_class_segment,
    ACTIONS(65), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    STATE(9), 5,
      sym_braced_expression,
      sym_attr_binding_braced,
      sym_attr_name_only,
      sym_tailwind_pair,
      sym_plain_class,
  [181] = 2,
    ACTIONS(82), 4,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      sym_element_tag,
    ACTIONS(21), 5,
      ts_builtin_sym_end,
      anon_sym_LF,
      sym_comment,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
  [195] = 4,
    ACTIONS(53), 1,
      anon_sym_LBRACE,
    ACTIONS(86), 1,
      aux_sym_plain_class_token1,
    STATE(13), 1,
      sym_braced_expression,
    ACTIONS(84), 5,
      ts_builtin_sym_end,
      anon_sym_LF,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [212] = 2,
    ACTIONS(90), 1,
      aux_sym_plain_class_token1,
    ACTIONS(88), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [224] = 2,
    ACTIONS(94), 1,
      aux_sym_plain_class_token1,
    ACTIONS(92), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [236] = 2,
    ACTIONS(98), 1,
      aux_sym_plain_class_token1,
    ACTIONS(96), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [248] = 2,
    ACTIONS(102), 1,
      aux_sym_plain_class_token1,
    ACTIONS(100), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [260] = 2,
    ACTIONS(106), 1,
      aux_sym_plain_class_token1,
    ACTIONS(104), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [272] = 2,
    ACTIONS(110), 1,
      aux_sym_plain_class_token1,
    ACTIONS(108), 6,
      ts_builtin_sym_end,
      anon_sym_LF,
      anon_sym_LBRACE,
      sym_hash_id,
      sym_attr_name_eq,
      aux_sym_tailwind_pair_token1,
  [284] = 3,
    ACTIONS(112), 1,
      anon_sym_DQUOTE,
    STATE(15), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(114), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [295] = 3,
    ACTIONS(117), 1,
      anon_sym_DQUOTE,
    STATE(17), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(119), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [306] = 3,
    ACTIONS(121), 1,
      anon_sym_DQUOTE,
    STATE(15), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(123), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [317] = 3,
    ACTIONS(47), 1,
      ts_builtin_sym_end,
    ACTIONS(125), 1,
      anon_sym_LF,
    STATE(7), 1,
      sym__eol,
  [327] = 3,
    ACTIONS(125), 1,
      anon_sym_LF,
    ACTIONS(127), 1,
      ts_builtin_sym_end,
    STATE(7), 1,
      sym__eol,
  [337] = 1,
    ACTIONS(129), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [342] = 1,
    ACTIONS(131), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [347] = 1,
    ACTIONS(133), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [352] = 1,
    ACTIONS(135), 2,
      anon_sym_GT,
      anon_sym_SLASH_GT,
  [357] = 1,
    ACTIONS(137), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [362] = 1,
    ACTIONS(139), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [367] = 2,
    ACTIONS(125), 1,
      anon_sym_LF,
    STATE(7), 1,
      sym__eol,
  [374] = 1,
    ACTIONS(141), 1,
      aux_sym_jsx_fragment_token1,
  [378] = 1,
    ACTIONS(143), 1,
      aux_sym_jsx_fragment_token1,
  [382] = 1,
    ACTIONS(145), 1,
      ts_builtin_sym_end,
  [386] = 1,
    ACTIONS(147), 1,
      aux_sym_fragment_section_token1,
  [390] = 1,
    ACTIONS(149), 1,
      sym_braced_body,
  [394] = 1,
    ACTIONS(135), 1,
      anon_sym_GT,
  [398] = 1,
    ACTIONS(151), 1,
      anon_sym_RBRACE,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(2)] = 0,
  [SMALL_STATE(3)] = 41,
  [SMALL_STATE(4)] = 82,
  [SMALL_STATE(5)] = 115,
  [SMALL_STATE(6)] = 148,
  [SMALL_STATE(7)] = 181,
  [SMALL_STATE(8)] = 195,
  [SMALL_STATE(9)] = 212,
  [SMALL_STATE(10)] = 224,
  [SMALL_STATE(11)] = 236,
  [SMALL_STATE(12)] = 248,
  [SMALL_STATE(13)] = 260,
  [SMALL_STATE(14)] = 272,
  [SMALL_STATE(15)] = 284,
  [SMALL_STATE(16)] = 295,
  [SMALL_STATE(17)] = 306,
  [SMALL_STATE(18)] = 317,
  [SMALL_STATE(19)] = 327,
  [SMALL_STATE(20)] = 337,
  [SMALL_STATE(21)] = 342,
  [SMALL_STATE(22)] = 347,
  [SMALL_STATE(23)] = 352,
  [SMALL_STATE(24)] = 357,
  [SMALL_STATE(25)] = 362,
  [SMALL_STATE(26)] = 367,
  [SMALL_STATE(27)] = 374,
  [SMALL_STATE(28)] = 378,
  [SMALL_STATE(29)] = 382,
  [SMALL_STATE(30)] = 386,
  [SMALL_STATE(31)] = 390,
  [SMALL_STATE(32)] = 394,
  [SMALL_STATE(33)] = 398,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0),
  [23] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(2),
  [26] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [29] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [32] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(30),
  [35] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(27),
  [38] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(28),
  [41] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [44] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [47] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 1, 0, 0),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [51] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 2, 0, 3),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [55] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [59] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [61] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [63] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 1, 0, 1),
  [65] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5),
  [67] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5), SHIFT_REPEAT(31),
  [70] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5), SHIFT_REPEAT(9),
  [73] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5), SHIFT_REPEAT(8),
  [76] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5), SHIFT_REPEAT(12),
  [79] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 5), SHIFT_REPEAT(11),
  [82] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0),
  [84] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attr_name_only, 1, 0, 0),
  [86] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_attr_name_only, 1, 0, 0),
  [88] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_class_segment, 1, 0, 0),
  [90] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_class_segment, 1, 0, 0),
  [92] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 1, 0, 2),
  [94] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_element_line_repeat1, 1, 0, 2),
  [96] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_plain_class, 1, 0, 0),
  [98] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_plain_class, 1, 0, 0),
  [100] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_tailwind_pair, 1, 0, 0),
  [102] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_tailwind_pair, 1, 0, 0),
  [104] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_attr_binding_braced, 2, 0, 4),
  [106] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_attr_binding_braced, 2, 0, 4),
  [108] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_braced_expression, 3, 0, 6),
  [110] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_braced_expression, 3, 0, 6),
  [112] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0),
  [114] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0), SHIFT_REPEAT(15),
  [117] = {.entry = {.count = 1, .reusable = false}}, SHIFT(25),
  [119] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [121] = {.entry = {.count = 1, .reusable = false}}, SHIFT(24),
  [123] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [125] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 2, 0, 0),
  [129] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_logical_line, 1, 0, 0),
  [131] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_jsx_fragment, 3, 0, 0),
  [133] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fragment_section, 2, 0, 0),
  [135] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 3, 0, 0),
  [139] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 2, 0, 0),
  [141] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [143] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [145] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [147] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [149] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [151] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_crepus(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
