#!/usr/bin/env python3
"""Generate crepuscularity-core Tailwind v4 default palette (hex) from theme.css."""

from __future__ import annotations

import math
import re
import sys
import urllib.request

THEME_URL = "https://raw.githubusercontent.com/tailwindlabs/tailwindcss/v4.3.0/packages/tailwindcss/theme.css"
OUT = "crates/crepuscularity-core/src/tailwind/colors.rs"


def oklch_to_srgb8(l: float, c: float, h_deg: float) -> tuple[int, int, int]:
    """OKLCH (0–1 L, C, hue deg) → sRGB 0–255. Matches CSS Color Level 4 pipeline."""
    h = math.radians(h_deg)
    a = c * math.cos(h)
    b = c * math.sin(h)

    l_ = l + 0.3963377774 * a + 0.2158037573 * b
    m_ = l - 0.1055613458 * a - 0.0638541728 * b
    s_ = l - 0.0894841775 * a - 1.2914855480 * b

    def lin_to_srgb(x: float) -> float:
        x = max(0.0, min(1.0, x))
        if x <= 0.0031308:
            return 12.92 * x
        return 1.055 * (x ** (1 / 2.4)) - 0.055

    def cbrt(x: float) -> float:
        return math.copysign(abs(x) ** (1 / 3), x)

    l_lin = cbrt(l_) ** 2
    m_lin = cbrt(m_) ** 2
    s_lin = cbrt(s_) ** 2

    r = +4.0767416621 * l_lin - 3.3077115913 * m_lin + 0.2309699292 * s_lin
    g = -1.2684380046 * l_lin + 2.6097574011 * m_lin - 0.3413193965 * s_lin
    b_ = -0.0041960863 * l_lin - 0.7034186147 * m_lin + 1.7076147010 * s_lin

    return (
        round(lin_to_srgb(r) * 255),
        round(lin_to_srgb(g) * 255),
        round(lin_to_srgb(b_) * 255),
    )


def parse_pct(s: str) -> float:
    s = s.strip()
    if s.endswith("%"):
        return float(s[:-1]) / 100.0
    return float(s)


def main() -> int:
    if len(sys.argv) > 2 and sys.argv[1] == "--theme":
        css = open(sys.argv[2], encoding="utf-8").read()
        out_path = sys.argv[3] if len(sys.argv) > 3 else OUT
    else:
        css = urllib.request.urlopen(THEME_URL, timeout=30).read().decode()
        out_path = sys.argv[1] if len(sys.argv) > 1 else OUT
    entries: list[tuple[str, str]] = []

    for m in re.finditer(
        r"--color-([a-z]+)-(\d+):\s*oklch\(([^)]+)\);", css
    ):
        family, shade, inner = m.group(1), m.group(2), m.group(3)
        parts = [p.strip() for p in inner.split()]
        l = parse_pct(parts[0])
        c = float(parts[1])
        h = float(parts[2])
        r, g, b = oklch_to_srgb8(l, c, h)
        entries.append((f"{family}-{shade}", f"#{r:02x}{g:02x}{b:02x}"))

    for m in re.finditer(r"--color-([a-z]+):\s*(#[0-9a-fA-F]+);", css):
        entries.append((m.group(1), m.group(2).lower()))

    entries.sort(key=lambda x: x[0])

    lines = [
        "//! Full Tailwind CSS v4 default palette (OKLCH → sRGB hex).",
        "//!",
        "//! Generated from tailwindcss v4.3.0 `theme.css` via `scripts/gen-tailwind-colors.py`.",
        "//! Class names (`bg-blue-500`, etc.) are unchanged; resolved hex values follow v4.",
        "",
        "/// Look up a Tailwind named color like `\"slate-500\"` or `\"red-100\"`.",
        "pub fn lookup_named_color(name: &str) -> Option<&'static str> {",
        "    Some(match name {",
    ]
    for key, hexv in entries:
        lines.append(f'        "{key}" => "{hexv}",')
    lines.extend(
        [
            '        _ => return None,',
            "    })",
            "}",
            "",
        ]
    )

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"wrote {len(entries)} colors to {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
