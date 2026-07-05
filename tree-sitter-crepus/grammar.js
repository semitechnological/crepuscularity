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
        repeat(field('class', $.class_segment)),
      ),

    // First token (element name, `if` / `else` / `for` / `include`, …).
    element_tag: ($) => token(/[^"<\s#\n]+/),

    // One “word” (no horizontal whitespace) after the tag: utilities, #ids, {expr}, etc.
    class_segment: ($) =>
      choice(
        $.braced_expression,
        $.hash_id,
        $.attr_binding_quoted,
        $.attr_binding_braced,
        $.attr_name_only,
        $.quoted,
        $.tailwind_pair,
        $.plain_class,
      ),

    // `{binding or expr}` (single-line; no `}` inside body).
    braced_expression: ($) =>
      seq('{', field('body', $.braced_body), '}'),

    braced_body: ($) => token(/[^}\n]*/),

    // `#hero` id shorthand on a line.
    hash_id: ($) => token(seq('#', /[a-zA-Z0-9_-]+/)),

    // `onclick={fn}` — name= immediately followed by `{…}`.
    // Prefer this over `attr_name_only` + a separate `braced_expression` segment.
    attr_binding_braced: ($) =>
      prec(
        2,
        seq(
          field('attr', $.attr_name_eq),
          field('value', $.braced_expression),
        ),
      ),

    attr_binding_quoted: ($) =>
      prec(
        2,
        seq(
          field('attr', $.attr_name_eq),
          field('value', $.quoted),
        ),
      ),

    // `data-x=` with no `{…}` on the same token.
    attr_name_only: ($) => prec(1, $.attr_name_eq),

    // High lexer precedence so `onclick=` wins over a `plain_class` slice that
    // would otherwise swallow `onclick={...}` as one token.
    attr_name_eq: ($) =>
      token(prec(2, seq(/[@a-zA-Z][a-zA-Z0-9_.-]*/, '='))),

    // `hover:bg-red-500` — single lexer atom so `w-full` is not mistaken for a
    // `variant` token that still needs a `:`. Utility’s first char is not `/`
    // so `https://…` in a segment stays `plain_class`.
    tailwind_pair: ($) =>
      prec(
        1,
        token(/[a-zA-Z0-9_\[\]-]+:[^"<\s\/\n][^"<\s\n]*/),
      ),

    // Fallback utility token (no structural `:` / `#` / `{` / `=` opener).
    plain_class: ($) => prec(-1, token(prec(-2, /[^"<\s\n]+/))),
  },
});
