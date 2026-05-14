from __future__ import annotations

import ctypes
import json
import os
import sys
from pathlib import Path
from typing import Any, Callable


def _default_lib() -> str:
    root = Path(__file__).resolve().parents[2]
    suffix = {"darwin": "dylib", "linux": "so", "win32": "dll"}.get(sys.platform, "so")
    name = "crepuscularity_abi.dll" if suffix == "dll" else f"libcrepuscularity_abi.{suffix}"
    return str(root / "target" / "debug" / name)


class CrepusAbiSession:
    def __init__(self, lib_path: str | None = None) -> None:
        self._lib = ctypes.CDLL(lib_path or os.environ.get("CREPUS_ABI_LIB") or _default_lib())
        self._callback_type = ctypes.CFUNCTYPE(None, ctypes.c_char_p, ctypes.c_void_p)
        self._callback_ref: Any = None
        self._configure()
        self._ptr = self._lib.crepus_session_new()
        if not self._ptr:
            raise RuntimeError(self._take_global_error())

    def close(self) -> None:
        if getattr(self, "_ptr", None):
            self._lib.crepus_session_free(self._ptr)
            self._ptr = None

    def __enter__(self) -> "CrepusAbiSession":
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def __del__(self) -> None:
        self.close()

    def set_template(self, template: str, base_dir: str | None = None) -> None:
        self._check(
            self._lib.crepus_session_set_template_string(
                self._ptr,
                template.encode(),
                base_dir.encode() if base_dir is not None else None,
            )
        )

    def set_context(self, context: dict[str, Any]) -> None:
        self._check(self._lib.crepus_session_set_context_json(self._ptr, json.dumps(context).encode()))

    def patch_context(self, context: dict[str, Any]) -> None:
        self._check(self._lib.crepus_session_apply_context_patch_json(self._ptr, json.dumps(context).encode()))

    def on_event(self, callback: Callable[[dict[str, Any]], None]) -> None:
        def trampoline(raw: bytes, _userdata: object) -> None:
            callback(json.loads(raw.decode()))

        self._callback_ref = self._callback_type(trampoline)
        self._check(self._lib.crepus_session_set_event_callback(self._ptr, self._callback_ref, None))

    def render_ir(self) -> dict[str, Any]:
        return self._take_json(self._lib.crepus_session_render_ir_json(self._ptr))

    def dispatch_event(self, event: str | dict[str, Any]) -> dict[str, Any]:
        raw = event if isinstance(event, str) else json.dumps(event)
        return self._take_json(self._lib.crepus_session_dispatch_event_json(self._ptr, raw.encode()))

    def _configure(self) -> None:
        self._lib.crepus_session_new.restype = ctypes.c_void_p
        self._lib.crepus_session_free.argtypes = [ctypes.c_void_p]
        self._lib.crepus_session_set_template_string.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
        self._lib.crepus_session_set_template_string.restype = ctypes.c_int32
        self._lib.crepus_session_set_context_json.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self._lib.crepus_session_set_context_json.restype = ctypes.c_int32
        self._lib.crepus_session_apply_context_patch_json.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self._lib.crepus_session_apply_context_patch_json.restype = ctypes.c_int32
        self._lib.crepus_session_set_event_callback.argtypes = [ctypes.c_void_p, self._callback_type, ctypes.c_void_p]
        self._lib.crepus_session_set_event_callback.restype = ctypes.c_int32
        self._lib.crepus_session_render_ir_json.argtypes = [ctypes.c_void_p]
        self._lib.crepus_session_render_ir_json.restype = ctypes.c_void_p
        self._lib.crepus_session_dispatch_event_json.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self._lib.crepus_session_dispatch_event_json.restype = ctypes.c_void_p
        self._lib.crepus_session_take_last_error.argtypes = [ctypes.c_void_p]
        self._lib.crepus_session_take_last_error.restype = ctypes.c_void_p
        self._lib.crepus_last_error.restype = ctypes.c_void_p
        self._lib.crepus_string_free.argtypes = [ctypes.c_void_p]

    def _check(self, code: int) -> None:
        if code != 0:
            raise RuntimeError(self._take_session_error())

    def _take_json(self, ptr: int) -> dict[str, Any]:
        if not ptr:
            raise RuntimeError(self._take_session_error())
        raw = ctypes.cast(ptr, ctypes.c_char_p).value or b""
        self._lib.crepus_string_free(ptr)
        return json.loads(raw.decode())

    def _take_session_error(self) -> str:
        ptr = self._lib.crepus_session_take_last_error(self._ptr)
        return self._take_string(ptr) if ptr else "crepuscularity ABI call failed"

    def _take_global_error(self) -> str:
        ptr = self._lib.crepus_last_error()
        return self._take_string(ptr) if ptr else "crepuscularity ABI call failed"

    def _take_string(self, ptr: int) -> str:
        raw = ctypes.cast(ptr, ctypes.c_char_p).value or b""
        self._lib.crepus_string_free(ptr)
        return raw.decode()
