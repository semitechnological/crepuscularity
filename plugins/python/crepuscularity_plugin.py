from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from html import escape


@dataclass
class ViewIr:
    version: int
    root: list[dict[str, Any]]


def _crepus_bin() -> str:
    return os.environ.get("CREPUS_BIN", "crepus")


def render_ir(path: str | Path, context: dict[str, Any] | None = None) -> ViewIr:
    args = [_crepus_bin(), "native", "ir", str(path)]
    input_data = None
    if context is not None:
        source = Path(path).read_text()
        payload = {"template": source, "context": context, "baseDir": str(Path(path).parent)}
        args = [_crepus_bin(), "native", "ir", "--stdin-json"]
        input_data = json.dumps(payload)
    proc = subprocess.run(args, input=input_data, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip())
    data = json.loads(proc.stdout)
    return ViewIr(version=data["version"], root=data["root"])


def render_html(path: str | Path, context: dict[str, Any] | None = None) -> str:
    return "".join(_render_node(node) for node in render_ir(path, context).root)


def _render_node(node: dict[str, Any]) -> str:
    kind = node.get("kind")
    if kind == "text":
        return escape(str(node.get("content", "")))
    if kind == "stack":
        axis = escape(str(node.get("axis", "column")))
        children = "".join(_render_node(child) for child in node.get("children", []))
        return f'<div data-crepus-kind="stack" data-axis="{axis}">{children}</div>'
    if kind == "scroll":
        axis = escape(str(node.get("axis", "column")))
        children = "".join(_render_node(child) for child in node.get("children", []))
        return f'<div data-crepus-kind="scroll" data-axis="{axis}">{children}</div>'
    if kind == "button":
        label = escape(str(node.get("label", "")))
        on_click = escape(str(node.get("onClick", "")))
        attr = f' data-onclick="{on_click}"' if on_click else ""
        return f"<button{attr}>{label}</button>"
    if kind == "image":
        src = escape(str(node.get("src", "")), quote=True)
        alt = escape(str(node.get("alt", "")), quote=True)
        return f'<img src="{src}" alt="{alt}">'
    if kind == "slotRotate":
        phrase = escape(str((node.get("phrases") or [""])[0]))
        return f'<span data-crepus-kind="slotRotate">{phrase}</span>'
    if kind == "input":
        placeholder = escape(str(node.get("placeholder", "")), quote=True)
        bind = escape(str(node.get("bind", "")), quote=True)
        if node.get("multiline"):
            return f'<textarea data-bind="{bind}" placeholder="{placeholder}"></textarea>'
        return f'<input data-bind="{bind}" placeholder="{placeholder}">'
    if kind == "picker":
        bind = escape(str(node.get("bind", "")), quote=True)
        options = "".join(
            f'<option value="{escape(str(opt.get("value", "")), quote=True)}">{escape(str(opt.get("label", "")))}</option>'
            for opt in node.get("options", [])
        )
        return f'<select data-bind="{bind}">{options}</select>'
    return ""
