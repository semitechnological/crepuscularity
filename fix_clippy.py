import sys

with open("crates/crepuscularity-runtime/src/styler.rs", "r") as f:
    content = f.read()

funcs = [
    "fn apply_layout",
    "fn apply_colors",
    "fn apply_typography",
    "fn apply_borders_shadows",
    "fn apply_misc"
]

for func in funcs:
    content = content.replace(func, f"#[allow(clippy::result_large_err)]\n{func}")

with open("crates/crepuscularity-runtime/src/styler.rs", "w") as f:
    f.write(content)
