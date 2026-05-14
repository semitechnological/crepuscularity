from __future__ import annotations

import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any


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
