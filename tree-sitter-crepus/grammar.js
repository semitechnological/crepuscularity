module.exports = grammar({
  name: 'crepus',

  // Horizontal whitespace only; newlines are explicit in `template` so one
  // `element_line` spans an entire logical line (tag + Tailwind classes).
  extras: ($) => [/[\t\f\v ]/],

  rules: {
    template: ($) =>
      seq(
        repeat(choice(seq($.logical_line, $._eol), $._eol)),
        optional($.logical_line),
      ),

    _eol: ($) => '\n',

    logical_line: ($) =>
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
      seq(
        field('tag', $.element_tag),
        repeat(field('class', $.element_class)),
      ),

    // First token (element name, `if` / `else` / `for` / `include`, …).
    element_tag: ($) => token(/[^"<\s#\n]+/),

    // Tailwind-style utilities and following tokens (`#id` is allowed here).
    element_class: ($) => token(/[^"<\s\n]+/),
  },
});
