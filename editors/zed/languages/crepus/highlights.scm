(comment) @comment

(quoted) @string

(jsx_fragment) @tag

(fragment_section) @keyword

(frontmatter_marker) @punctuation.special

(element_line (element_tag) @tag)

; Current pinned grammar exposes one `element_class` token for post-tag
; segments. Use regex predicates to color common forms differently.
((element_class) @keyword
  (#match? @keyword "^\\{[^}]*\\}$"))

((element_class) @label
  (#match? @label "^#[A-Za-z0-9_-]+$"))

((element_class) @property
  (#match? @property "^[@A-Za-z][A-Za-z0-9_.-]*=.*$"))

((element_class) @type
  (#match? @type "^[A-Za-z0-9_\\[\\]-]+:[^[:space:]].*$"))

((element_class) @attribute
  (#not-match? @attribute "^\\{[^}]*\\}$")
  (#not-match? @attribute "^#[A-Za-z0-9_-]+$")
  (#not-match? @attribute "^[@A-Za-z][A-Za-z0-9_.-]*=.*$")
  (#not-match? @attribute "^[A-Za-z0-9_\\[\\]-]+:[^[:space:]].*$"))
