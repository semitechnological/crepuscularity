module.exports = grammar({
  name: 'crepusx',

  // JSX-style source: whitespace (including newlines) is ignored between tokens.
  extras: ($) => [/\s/],

  rules: {
    source: ($) => repeat($._node),

    _node: ($) =>
      choice(
        $.comment,
        $.jsx_element,
        $.jsx_self_closing,
        $.jsx_fragment,
        $.braced_expression,
        $.text,
      ),

    comment: ($) => token(seq('#', /[^\n]*/)),

    jsx_element: ($) =>
      seq(
        $.jsx_open_tag,
        repeat($._node),
        $.jsx_close_tag,
      ),

    // JSX tag name: components (CamelCase), intrinsic tags (div, span) and dotted names.
    identifier: ($) => token.immediate(/[a-zA-Z_][a-zA-Z0-9_:.]*/),

    jsx_self_closing: ($) =>
      seq(
        '<',
        $.identifier,
        repeat($.attribute),
        '/>',
      ),

    jsx_fragment: ($) =>
      seq(
        '<>',
        repeat($._node),
        '</>',
      ),

    jsx_open_tag: ($) =>
      seq(
        '<',
        $.identifier,
        repeat($.attribute),
        '>',
      ),

    jsx_close_tag: ($) =>
      seq(
        '</',
        $.identifier,
        '>',
      ),

    attribute: ($) =>
      choice(
        $.attr_binding_quoted,
        $.attr_binding_braced,
        $.attr_name_only,
      ),

    attr_binding_quoted: ($) =>
      seq(
        $.attr_name,
        '=',
        $.quoted,
      ),

    attr_binding_braced: ($) =>
      seq(
        $.attr_name,
        '=',
        $.braced_expression,
      ),

    attr_name_only: ($) => $.attr_name,

    attr_name: ($) => token(/[a-z][a-zA-Z0-9_:-]*/),

    quoted: ($) =>
      seq(
        '"',
        repeat(
          choice(
            token(prec(1, /\\./)),
            /[^"\\]/,
          ),
        ),
        '"',
      ),

    braced_expression: ($) =>
      seq(
        '{',
        optional($.braced_body),
        '}',
      ),

    braced_body: ($) =>
      repeat1(
        choice(
          $.braced_expression,
          token(/[^}{]+/),
        ),
      ),

    text: ($) => token(/[^<#{}]+/),
  },
});
