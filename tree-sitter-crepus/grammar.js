module.exports = grammar({
  name: 'crepus',

  extras: ($) => [/\s/],

  rules: {
    template: ($) => repeat(choice($._node, $._eol)),
    _eol: ($) => '\n',

    _node: ($) =>
      choice(
        $.comment,
        $.fragment_section,
        $.frontmatter_marker,
        $.jsx_fragment,
        $.quoted,
        $.element_line,
      ),

    comment: ($) => token(seq('#', /[^\n]*/)),
    frontmatter_marker: ($) => token('+++'),
    fragment_section: ($) => seq('---', /[^\n]+/),

    jsx_fragment: ($) =>
      choice(
        seq('<', /[^>\n]+/, '>'),
        seq('</', /[^>\n]+/, '>'),
        seq('<', /[^>\n]+/, '/>'),
      ),

    quoted: ($) =>
      seq(
        '"',
        repeat(choice(token(prec(1, /\\./)), /[^"\\]/)),
        '"',
      ),

    element_line: ($) =>
      prec(
        -1,
        seq(repeat1(/[^"<\n#]/), optional(seq(/\s+/, repeat(/[^<\n"]/)))),
      ),
  },
});
