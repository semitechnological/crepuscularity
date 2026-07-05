(comment) @comment

(quoted) @string.special

(jsx_fragment) @tag

(fragment_section) @keyword

(frontmatter_marker) @punctuation.special

(element_line (element_tag) @tag)

(braced_expression) @keyword

(hash_id) @label

(attr_binding_braced
  (attr_name_eq) @function
  (braced_expression) @keyword)

(attr_name_only) @function

(tailwind_pair) @type

((plain_class) @keyword
  (#match? @keyword "\\(\\.\\.\\.\\)"))

((plain_class) @keyword
  (#match? @keyword "^\\{[^}]*\\}$"))

((plain_class) @label
  (#match? @label "^#[A-Za-z0-9_-]+$"))

((plain_class) @function
  (#match? @function "^[@A-Za-z][A-Za-z0-9_.-]*=.*$"))

((plain_class) @type
  (#match? @type "^[A-Za-z0-9_\\[\\]-]+:[^[:space:]].*$"))

((plain_class) @attribute
  (#not-match? @attribute "^\\{[^}]*\\}$")
  (#not-match? @attribute "^#[A-Za-z0-9_-]+$")
  (#not-match? @attribute "^[@A-Za-z][A-Za-z0-9_.-]*=.*$")
  (#not-match? @attribute "^[A-Za-z0-9_\\[\\]-]+:[^[:space:]].*$"))
