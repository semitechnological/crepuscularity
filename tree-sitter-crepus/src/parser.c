#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 26
#define LARGE_STATE_COUNT 4
#define SYMBOL_COUNT 26
#define ALIAS_COUNT 0
#define TOKEN_COUNT 16
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 2
#define MAX_ALIAS_SEQUENCE_LENGTH 3
#define PRODUCTION_ID_COUNT 5

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
  sym_element_class = 15,
  sym_template = 16,
  sym__eol = 17,
  sym_logical_line = 18,
  sym_fragment_section = 19,
  sym_jsx_fragment = 20,
  sym_quoted = 21,
  sym_element_line = 22,
  aux_sym_template_repeat1 = 23,
  aux_sym_quoted_repeat1 = 24,
  aux_sym_element_line_repeat1 = 25,
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
  [sym_element_class] = "element_class",
  [sym_template] = "template",
  [sym__eol] = "_eol",
  [sym_logical_line] = "logical_line",
  [sym_fragment_section] = "fragment_section",
  [sym_jsx_fragment] = "jsx_fragment",
  [sym_quoted] = "quoted",
  [sym_element_line] = "element_line",
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
  [sym_element_class] = sym_element_class,
  [sym_template] = sym_template,
  [sym__eol] = sym__eol,
  [sym_logical_line] = sym_logical_line,
  [sym_fragment_section] = sym_fragment_section,
  [sym_jsx_fragment] = sym_jsx_fragment,
  [sym_quoted] = sym_quoted,
  [sym_element_line] = sym_element_line,
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
  [sym_element_class] = {
    .visible = true,
    .named = true,
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
  field_class = 1,
  field_tag = 2,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_class] = "class",
  [field_tag] = "tag",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
  [4] = {.index = 4, .length = 2},
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
    {field_class, 0, .inherited = true},
    {field_class, 1, .inherited = true},
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
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(9);
      ADVANCE_MAP(
        '\n', 10,
        '"', 24,
        '#', 11,
        '+', 29,
        '-', 31,
        '/', 32,
        '<', 16,
        '>', 20,
      );
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(0);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(33);
      END_STATE();
    case 1:
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(27);
      if (lookahead != 0) ADVANCE(26);
      END_STATE();
    case 2:
      if (lookahead == '/') ADVANCE(3);
      if (lookahead == '>') ADVANCE(19);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') SKIP(2);
      END_STATE();
    case 3:
      if (lookahead == '>') ADVANCE(22);
      END_STATE();
    case 4:
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(14);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead)) ADVANCE(15);
      END_STATE();
    case 5:
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(17);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead) &&
          lookahead != '>') ADVANCE(18);
      END_STATE();
    case 6:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(25);
      END_STATE();
    case 7:
      if (eof) ADVANCE(9);
      if (lookahead == '\n') ADVANCE(10);
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '#') ADVANCE(11);
      if (lookahead == '+') ADVANCE(29);
      if (lookahead == '-') ADVANCE(31);
      if (lookahead == '<') ADVANCE(16);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(7);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(33);
      END_STATE();
    case 8:
      if (eof) ADVANCE(9);
      if (lookahead == '\n') ADVANCE(10);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') SKIP(8);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(34);
      END_STATE();
    case 9:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 10:
      ACCEPT_TOKEN(anon_sym_LF);
      END_STATE();
    case 11:
      ACCEPT_TOKEN(sym_comment);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(11);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(sym_frontmatter_marker);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 13:
      ACCEPT_TOKEN(anon_sym_DASH_DASH_DASH);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 14:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(14);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead)) ADVANCE(15);
      END_STATE();
    case 15:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(15);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '/') ADVANCE(21);
      END_STATE();
    case 17:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(17);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\f' < lookahead) &&
          lookahead != '>') ADVANCE(18);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '>') ADVANCE(18);
      END_STATE();
    case 19:
      ACCEPT_TOKEN(anon_sym_GT);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(anon_sym_LT_SLASH);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(anon_sym_SLASH_GT);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(anon_sym_SLASH_GT);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 24:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 25:
      ACCEPT_TOKEN(aux_sym_quoted_token1);
      END_STATE();
    case 26:
      ACCEPT_TOKEN(aux_sym_quoted_token2);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(aux_sym_quoted_token2);
      if (lookahead == '\\') ADVANCE(6);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(27);
      if (lookahead != 0 &&
          lookahead != '"') ADVANCE(26);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '+') ADVANCE(12);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '+') ADVANCE(28);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '-') ADVANCE(13);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '-') ADVANCE(30);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead == '>') ADVANCE(23);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(sym_element_tag);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(33);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(sym_element_class);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != ' ' &&
          lookahead != '"' &&
          lookahead != '<') ADVANCE(34);
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
  [4] = {.lex_state = 7},
  [5] = {.lex_state = 1},
  [6] = {.lex_state = 8},
  [7] = {.lex_state = 1},
  [8] = {.lex_state = 1},
  [9] = {.lex_state = 8},
  [10] = {.lex_state = 8},
  [11] = {.lex_state = 0},
  [12] = {.lex_state = 0},
  [13] = {.lex_state = 8},
  [14] = {.lex_state = 2},
  [15] = {.lex_state = 0},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 0},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 0},
  [20] = {.lex_state = 0},
  [21] = {.lex_state = 4},
  [22] = {.lex_state = 0},
  [23] = {.lex_state = 5},
  [24] = {.lex_state = 5},
  [25] = {.lex_state = 2},
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
    [sym_element_tag] = ACTIONS(1),
  },
  [1] = {
    [sym_template] = STATE(22),
    [sym__eol] = STATE(2),
    [sym_logical_line] = STATE(11),
    [sym_fragment_section] = STATE(16),
    [sym_jsx_fragment] = STATE(16),
    [sym_quoted] = STATE(16),
    [sym_element_line] = STATE(16),
    [aux_sym_template_repeat1] = STATE(2),
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
  [2] = {
    [sym__eol] = STATE(3),
    [sym_logical_line] = STATE(12),
    [sym_fragment_section] = STATE(16),
    [sym_jsx_fragment] = STATE(16),
    [sym_quoted] = STATE(16),
    [sym_element_line] = STATE(16),
    [aux_sym_template_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(21),
    [anon_sym_LF] = ACTIONS(23),
    [sym_comment] = ACTIONS(7),
    [sym_frontmatter_marker] = ACTIONS(9),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(11),
    [anon_sym_LT] = ACTIONS(13),
    [anon_sym_LT_SLASH] = ACTIONS(15),
    [anon_sym_DQUOTE] = ACTIONS(17),
    [sym_element_tag] = ACTIONS(19),
  },
  [3] = {
    [sym__eol] = STATE(3),
    [sym_logical_line] = STATE(17),
    [sym_fragment_section] = STATE(16),
    [sym_jsx_fragment] = STATE(16),
    [sym_quoted] = STATE(16),
    [sym_element_line] = STATE(16),
    [aux_sym_template_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(25),
    [anon_sym_LF] = ACTIONS(27),
    [sym_comment] = ACTIONS(30),
    [sym_frontmatter_marker] = ACTIONS(33),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(36),
    [anon_sym_LT] = ACTIONS(39),
    [anon_sym_LT_SLASH] = ACTIONS(42),
    [anon_sym_DQUOTE] = ACTIONS(45),
    [sym_element_tag] = ACTIONS(48),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 2,
    ACTIONS(51), 4,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      sym_element_tag,
    ACTIONS(25), 5,
      ts_builtin_sym_end,
      anon_sym_LF,
      sym_comment,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
  [14] = 3,
    ACTIONS(53), 1,
      anon_sym_DQUOTE,
    STATE(8), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(55), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [25] = 3,
    ACTIONS(59), 1,
      sym_element_class,
    STATE(9), 1,
      aux_sym_element_line_repeat1,
    ACTIONS(57), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [36] = 3,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    STATE(7), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(63), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [47] = 3,
    ACTIONS(66), 1,
      anon_sym_DQUOTE,
    STATE(7), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(68), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [58] = 3,
    ACTIONS(59), 1,
      sym_element_class,
    STATE(10), 1,
      aux_sym_element_line_repeat1,
    ACTIONS(70), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [69] = 3,
    ACTIONS(74), 1,
      sym_element_class,
    STATE(10), 1,
      aux_sym_element_line_repeat1,
    ACTIONS(72), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [80] = 3,
    ACTIONS(21), 1,
      ts_builtin_sym_end,
    ACTIONS(77), 1,
      anon_sym_LF,
    STATE(4), 1,
      sym__eol,
  [90] = 3,
    ACTIONS(77), 1,
      anon_sym_LF,
    ACTIONS(79), 1,
      ts_builtin_sym_end,
    STATE(4), 1,
      sym__eol,
  [100] = 1,
    ACTIONS(81), 3,
      ts_builtin_sym_end,
      anon_sym_LF,
      sym_element_class,
  [106] = 1,
    ACTIONS(83), 2,
      anon_sym_GT,
      anon_sym_SLASH_GT,
  [111] = 1,
    ACTIONS(85), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [116] = 1,
    ACTIONS(87), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [121] = 2,
    ACTIONS(77), 1,
      anon_sym_LF,
    STATE(4), 1,
      sym__eol,
  [128] = 1,
    ACTIONS(89), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [133] = 1,
    ACTIONS(91), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [138] = 1,
    ACTIONS(93), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
  [143] = 1,
    ACTIONS(95), 1,
      aux_sym_fragment_section_token1,
  [147] = 1,
    ACTIONS(97), 1,
      ts_builtin_sym_end,
  [151] = 1,
    ACTIONS(99), 1,
      aux_sym_jsx_fragment_token1,
  [155] = 1,
    ACTIONS(101), 1,
      aux_sym_jsx_fragment_token1,
  [159] = 1,
    ACTIONS(83), 1,
      anon_sym_GT,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(4)] = 0,
  [SMALL_STATE(5)] = 14,
  [SMALL_STATE(6)] = 25,
  [SMALL_STATE(7)] = 36,
  [SMALL_STATE(8)] = 47,
  [SMALL_STATE(9)] = 58,
  [SMALL_STATE(10)] = 69,
  [SMALL_STATE(11)] = 80,
  [SMALL_STATE(12)] = 90,
  [SMALL_STATE(13)] = 100,
  [SMALL_STATE(14)] = 106,
  [SMALL_STATE(15)] = 111,
  [SMALL_STATE(16)] = 116,
  [SMALL_STATE(17)] = 121,
  [SMALL_STATE(18)] = 128,
  [SMALL_STATE(19)] = 133,
  [SMALL_STATE(20)] = 138,
  [SMALL_STATE(21)] = 143,
  [SMALL_STATE(22)] = 147,
  [SMALL_STATE(23)] = 151,
  [SMALL_STATE(24)] = 155,
  [SMALL_STATE(25)] = 159,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(21),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 1, 0, 0),
  [23] = {.entry = {.count = 1, .reusable = true}}, SHIFT(3),
  [25] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0),
  [27] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(3),
  [30] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [33] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(16),
  [36] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(21),
  [39] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(23),
  [42] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(24),
  [45] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [48] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(6),
  [51] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [57] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 1, 0, 1),
  [59] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [61] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0),
  [63] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0), SHIFT_REPEAT(7),
  [66] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [68] = {.entry = {.count = 1, .reusable = false}}, SHIFT(7),
  [70] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 2, 0, 3),
  [72] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 4),
  [74] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 4), SHIFT_REPEAT(13),
  [77] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [79] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 2, 0, 0),
  [81] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 1, 0, 2),
  [83] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [85] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fragment_section, 2, 0, 0),
  [87] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_logical_line, 1, 0, 0),
  [89] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 3, 0, 0),
  [91] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 2, 0, 0),
  [93] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_jsx_fragment, 3, 0, 0),
  [95] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [97] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [99] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
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
