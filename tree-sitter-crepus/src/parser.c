#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 22
#define LARGE_STATE_COUNT 4
#define SYMBOL_COUNT 28
#define ALIAS_COUNT 0
#define TOKEN_COUNT 17
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 0
#define MAX_ALIAS_SEQUENCE_LENGTH 3
#define PRODUCTION_ID_COUNT 1

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
  aux_sym_element_line_token1 = 14,
  aux_sym_element_line_token2 = 15,
  aux_sym_element_line_token3 = 16,
  sym_template = 17,
  sym__eol = 18,
  sym__node = 19,
  sym_fragment_section = 20,
  sym_jsx_fragment = 21,
  sym_quoted = 22,
  sym_element_line = 23,
  aux_sym_template_repeat1 = 24,
  aux_sym_quoted_repeat1 = 25,
  aux_sym_element_line_repeat1 = 26,
  aux_sym_element_line_repeat2 = 27,
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
  [aux_sym_element_line_token1] = "element_line_token1",
  [aux_sym_element_line_token2] = "element_line_token2",
  [aux_sym_element_line_token3] = "element_line_token3",
  [sym_template] = "template",
  [sym__eol] = "_eol",
  [sym__node] = "_node",
  [sym_fragment_section] = "fragment_section",
  [sym_jsx_fragment] = "jsx_fragment",
  [sym_quoted] = "quoted",
  [sym_element_line] = "element_line",
  [aux_sym_template_repeat1] = "template_repeat1",
  [aux_sym_quoted_repeat1] = "quoted_repeat1",
  [aux_sym_element_line_repeat1] = "element_line_repeat1",
  [aux_sym_element_line_repeat2] = "element_line_repeat2",
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
  [aux_sym_element_line_token1] = aux_sym_element_line_token1,
  [aux_sym_element_line_token2] = aux_sym_element_line_token2,
  [aux_sym_element_line_token3] = aux_sym_element_line_token3,
  [sym_template] = sym_template,
  [sym__eol] = sym__eol,
  [sym__node] = sym__node,
  [sym_fragment_section] = sym_fragment_section,
  [sym_jsx_fragment] = sym_jsx_fragment,
  [sym_quoted] = sym_quoted,
  [sym_element_line] = sym_element_line,
  [aux_sym_template_repeat1] = aux_sym_template_repeat1,
  [aux_sym_quoted_repeat1] = aux_sym_quoted_repeat1,
  [aux_sym_element_line_repeat1] = aux_sym_element_line_repeat1,
  [aux_sym_element_line_repeat2] = aux_sym_element_line_repeat2,
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
  [aux_sym_element_line_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_element_line_token2] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_element_line_token3] = {
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
  [sym__node] = {
    .visible = false,
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
  [aux_sym_element_line_repeat2] = {
    .visible = false,
    .named = false,
  },
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
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(11);
      ADVANCE_MAP(
        '"', 24,
        '#', 13,
        '+', 5,
        '-', 7,
        '/', 8,
        '<', 18,
        '>', 21,
        '\\', 9,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      END_STATE();
    case 1:
      if (lookahead == '\n') SKIP(1);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(19);
      if (lookahead != 0 &&
          lookahead != '>') ADVANCE(20);
      END_STATE();
    case 2:
      if (lookahead == '\n') SKIP(2);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(16);
      if (lookahead != 0) ADVANCE(17);
      END_STATE();
    case 3:
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '\\') ADVANCE(9);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(27);
      if (lookahead != 0) ADVANCE(26);
      END_STATE();
    case 4:
      if (lookahead == '+') ADVANCE(14);
      END_STATE();
    case 5:
      if (lookahead == '+') ADVANCE(4);
      END_STATE();
    case 6:
      if (lookahead == '-') ADVANCE(15);
      END_STATE();
    case 7:
      if (lookahead == '-') ADVANCE(6);
      END_STATE();
    case 8:
      if (lookahead == '>') ADVANCE(23);
      END_STATE();
    case 9:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(25);
      END_STATE();
    case 10:
      if (eof) ADVANCE(11);
      if (lookahead == '\n') ADVANCE(12);
      if (lookahead == '"') ADVANCE(24);
      if (lookahead == '#') ADVANCE(13);
      if (lookahead == '+') ADVANCE(30);
      if (lookahead == '-') ADVANCE(31);
      if (lookahead == '<') ADVANCE(18);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(29);
      if (lookahead != 0) ADVANCE(28);
      END_STATE();
    case 11:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 12:
      ACCEPT_TOKEN(anon_sym_LF);
      if (lookahead == '\n') ADVANCE(12);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(29);
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
      ACCEPT_TOKEN(anon_sym_DASH_DASH_DASH);
      END_STATE();
    case 16:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(16);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(17);
      END_STATE();
    case 17:
      ACCEPT_TOKEN(aux_sym_fragment_section_token1);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(17);
      END_STATE();
    case 18:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '/') ADVANCE(22);
      END_STATE();
    case 19:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(19);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '>') ADVANCE(20);
      END_STATE();
    case 20:
      ACCEPT_TOKEN(aux_sym_jsx_fragment_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '>') ADVANCE(20);
      END_STATE();
    case 21:
      ACCEPT_TOKEN(anon_sym_GT);
      END_STATE();
    case 22:
      ACCEPT_TOKEN(anon_sym_LT_SLASH);
      END_STATE();
    case 23:
      ACCEPT_TOKEN(anon_sym_SLASH_GT);
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
      if (lookahead == '\\') ADVANCE(9);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(27);
      if (lookahead != 0 &&
          lookahead != '"') ADVANCE(26);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(aux_sym_element_line_token1);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(aux_sym_element_line_token1);
      if (lookahead == '\n') ADVANCE(12);
      if (lookahead == '+') ADVANCE(30);
      if (lookahead == '-') ADVANCE(31);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(29);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '#' &&
          lookahead != '<') ADVANCE(28);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(aux_sym_element_line_token1);
      if (lookahead == '+') ADVANCE(4);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(aux_sym_element_line_token1);
      if (lookahead == '-') ADVANCE(6);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 10},
  [2] = {.lex_state = 10},
  [3] = {.lex_state = 10},
  [4] = {.lex_state = 10},
  [5] = {.lex_state = 10},
  [6] = {.lex_state = 10},
  [7] = {.lex_state = 10},
  [8] = {.lex_state = 10},
  [9] = {.lex_state = 10},
  [10] = {.lex_state = 10},
  [11] = {.lex_state = 10},
  [12] = {.lex_state = 10},
  [13] = {.lex_state = 3},
  [14] = {.lex_state = 3},
  [15] = {.lex_state = 3},
  [16] = {.lex_state = 0},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 0},
  [19] = {.lex_state = 2},
  [20] = {.lex_state = 1},
  [21] = {.lex_state = 0},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [sym_comment] = ACTIONS(1),
    [sym_frontmatter_marker] = ACTIONS(1),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_LT_SLASH] = ACTIONS(1),
    [anon_sym_SLASH_GT] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [aux_sym_quoted_token1] = ACTIONS(1),
  },
  [1] = {
    [sym_template] = STATE(18),
    [sym__eol] = STATE(2),
    [sym__node] = STATE(2),
    [sym_fragment_section] = STATE(2),
    [sym_jsx_fragment] = STATE(2),
    [sym_quoted] = STATE(2),
    [sym_element_line] = STATE(2),
    [aux_sym_template_repeat1] = STATE(2),
    [aux_sym_element_line_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_LF] = ACTIONS(5),
    [sym_comment] = ACTIONS(5),
    [sym_frontmatter_marker] = ACTIONS(5),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(7),
    [anon_sym_LT] = ACTIONS(9),
    [anon_sym_LT_SLASH] = ACTIONS(11),
    [anon_sym_DQUOTE] = ACTIONS(13),
    [aux_sym_element_line_token1] = ACTIONS(15),
  },
  [2] = {
    [sym__eol] = STATE(3),
    [sym__node] = STATE(3),
    [sym_fragment_section] = STATE(3),
    [sym_jsx_fragment] = STATE(3),
    [sym_quoted] = STATE(3),
    [sym_element_line] = STATE(3),
    [aux_sym_template_repeat1] = STATE(3),
    [aux_sym_element_line_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(17),
    [anon_sym_LF] = ACTIONS(19),
    [sym_comment] = ACTIONS(19),
    [sym_frontmatter_marker] = ACTIONS(19),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(7),
    [anon_sym_LT] = ACTIONS(9),
    [anon_sym_LT_SLASH] = ACTIONS(11),
    [anon_sym_DQUOTE] = ACTIONS(13),
    [aux_sym_element_line_token1] = ACTIONS(15),
  },
  [3] = {
    [sym__eol] = STATE(3),
    [sym__node] = STATE(3),
    [sym_fragment_section] = STATE(3),
    [sym_jsx_fragment] = STATE(3),
    [sym_quoted] = STATE(3),
    [sym_element_line] = STATE(3),
    [aux_sym_template_repeat1] = STATE(3),
    [aux_sym_element_line_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(21),
    [anon_sym_LF] = ACTIONS(23),
    [sym_comment] = ACTIONS(23),
    [sym_frontmatter_marker] = ACTIONS(23),
    [anon_sym_DASH_DASH_DASH] = ACTIONS(26),
    [anon_sym_LT] = ACTIONS(29),
    [anon_sym_LT_SLASH] = ACTIONS(32),
    [anon_sym_DQUOTE] = ACTIONS(35),
    [aux_sym_element_line_token1] = ACTIONS(38),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 5,
    ACTIONS(41), 1,
      ts_builtin_sym_end,
    ACTIONS(45), 1,
      aux_sym_element_line_token1,
    ACTIONS(47), 1,
      aux_sym_element_line_token2,
    STATE(6), 1,
      aux_sym_element_line_repeat1,
    ACTIONS(43), 7,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
  [22] = 4,
    ACTIONS(49), 1,
      ts_builtin_sym_end,
    ACTIONS(53), 1,
      aux_sym_element_line_token3,
    STATE(7), 1,
      aux_sym_element_line_repeat2,
    ACTIONS(51), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [42] = 4,
    ACTIONS(55), 1,
      ts_builtin_sym_end,
    ACTIONS(59), 1,
      aux_sym_element_line_token1,
    STATE(6), 1,
      aux_sym_element_line_repeat1,
    ACTIONS(57), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token2,
  [62] = 4,
    ACTIONS(62), 1,
      ts_builtin_sym_end,
    ACTIONS(66), 1,
      aux_sym_element_line_token3,
    STATE(8), 1,
      aux_sym_element_line_repeat2,
    ACTIONS(64), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [82] = 4,
    ACTIONS(68), 1,
      ts_builtin_sym_end,
    ACTIONS(72), 1,
      aux_sym_element_line_token3,
    STATE(8), 1,
      aux_sym_element_line_repeat2,
    ACTIONS(70), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [102] = 2,
    ACTIONS(75), 1,
      ts_builtin_sym_end,
    ACTIONS(77), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [116] = 2,
    ACTIONS(79), 1,
      ts_builtin_sym_end,
    ACTIONS(81), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [130] = 2,
    ACTIONS(83), 1,
      ts_builtin_sym_end,
    ACTIONS(85), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [144] = 2,
    ACTIONS(87), 1,
      ts_builtin_sym_end,
    ACTIONS(89), 8,
      anon_sym_LF,
      sym_comment,
      sym_frontmatter_marker,
      anon_sym_DASH_DASH_DASH,
      anon_sym_LT,
      anon_sym_LT_SLASH,
      anon_sym_DQUOTE,
      aux_sym_element_line_token1,
  [158] = 3,
    ACTIONS(91), 1,
      anon_sym_DQUOTE,
    STATE(15), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(93), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [169] = 3,
    ACTIONS(95), 1,
      anon_sym_DQUOTE,
    STATE(14), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(97), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [180] = 3,
    ACTIONS(100), 1,
      anon_sym_DQUOTE,
    STATE(14), 1,
      aux_sym_quoted_repeat1,
    ACTIONS(102), 2,
      aux_sym_quoted_token1,
      aux_sym_quoted_token2,
  [191] = 1,
    ACTIONS(104), 2,
      anon_sym_GT,
      anon_sym_SLASH_GT,
  [196] = 1,
    ACTIONS(106), 1,
      aux_sym_jsx_fragment_token1,
  [200] = 1,
    ACTIONS(108), 1,
      ts_builtin_sym_end,
  [204] = 1,
    ACTIONS(110), 1,
      aux_sym_fragment_section_token1,
  [208] = 1,
    ACTIONS(112), 1,
      aux_sym_jsx_fragment_token1,
  [212] = 1,
    ACTIONS(104), 1,
      anon_sym_GT,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(4)] = 0,
  [SMALL_STATE(5)] = 22,
  [SMALL_STATE(6)] = 42,
  [SMALL_STATE(7)] = 62,
  [SMALL_STATE(8)] = 82,
  [SMALL_STATE(9)] = 102,
  [SMALL_STATE(10)] = 116,
  [SMALL_STATE(11)] = 130,
  [SMALL_STATE(12)] = 144,
  [SMALL_STATE(13)] = 158,
  [SMALL_STATE(14)] = 169,
  [SMALL_STATE(15)] = 180,
  [SMALL_STATE(16)] = 191,
  [SMALL_STATE(17)] = 196,
  [SMALL_STATE(18)] = 200,
  [SMALL_STATE(19)] = 204,
  [SMALL_STATE(20)] = 208,
  [SMALL_STATE(21)] = 212,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = false}}, SHIFT(2),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(19),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(17),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [17] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_template, 1, 0, 0),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(3),
  [21] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0),
  [23] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(3),
  [26] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(19),
  [29] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(17),
  [32] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [35] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(13),
  [38] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_template_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [41] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 1, 0, 0),
  [43] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_element_line, 1, 0, 0),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(5),
  [49] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 2, 0, 0),
  [51] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_element_line, 2, 0, 0),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(7),
  [55] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 0),
  [57] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 0),
  [59] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_element_line_repeat1, 2, 0, 0), SHIFT_REPEAT(6),
  [62] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_element_line, 3, 0, 0),
  [64] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_element_line, 3, 0, 0),
  [66] = {.entry = {.count = 1, .reusable = false}}, SHIFT(8),
  [68] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_element_line_repeat2, 2, 0, 0),
  [70] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_element_line_repeat2, 2, 0, 0),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_element_line_repeat2, 2, 0, 0), SHIFT_REPEAT(8),
  [75] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fragment_section, 2, 0, 0),
  [77] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fragment_section, 2, 0, 0),
  [79] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_jsx_fragment, 3, 0, 0),
  [81] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_jsx_fragment, 3, 0, 0),
  [83] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 2, 0, 0),
  [85] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_quoted, 2, 0, 0),
  [87] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_quoted, 3, 0, 0),
  [89] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_quoted, 3, 0, 0),
  [91] = {.entry = {.count = 1, .reusable = false}}, SHIFT(11),
  [93] = {.entry = {.count = 1, .reusable = false}}, SHIFT(15),
  [95] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0),
  [97] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_quoted_repeat1, 2, 0, 0), SHIFT_REPEAT(14),
  [100] = {.entry = {.count = 1, .reusable = false}}, SHIFT(12),
  [102] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [104] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [106] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [108] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [110] = {.entry = {.count = 1, .reusable = true}}, SHIFT(9),
  [112] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
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
