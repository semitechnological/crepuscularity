package crepuscularity

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCrepusBinValidation(t *testing.T) {
	orig := os.Getenv("CREPUS_BIN")
	defer os.Setenv("CREPUS_BIN", orig)

	// test absolute path
	absPath := "/usr/bin/crepus"
	if filepath.Separator == '\\' {
		absPath = "C:\\bin\\crepus"
	}
	os.Setenv("CREPUS_BIN", absPath)
	if crepusBin() != absPath {
		t.Fatalf("expected %s, got %s", absPath, crepusBin())
	}

	// test simple binary name valid
	simple := "crepus"
	os.Setenv("CREPUS_BIN", simple)
	if crepusBin() != simple {
		t.Fatalf("expected %s, got %s", simple, crepusBin())
	}

	// test invalid binary name panics
	os.Setenv("CREPUS_BIN", "mycrepus")
	assertPanic(t, func() { crepusBin() })

	// test relative path panics
	os.Setenv("CREPUS_BIN", "./crepus")
	assertPanic(t, func() { crepusBin() })
}

func assertPanic(t *testing.T, f func()) {
	defer func() {
		if r := recover(); r == nil {
			t.Errorf("The code did not panic")
		}
	}()
	f()
}

func TestRenderIR(t *testing.T) {
	os.Setenv("CREPUS_ALLOWED_DIR", "..")
	ir, err := RenderIR("../fixtures/hello.crepus", map[string]any{"name": "Ada"})
	if err != nil {
		t.Fatal(err)
	}
	if ir.Version != 4 {
		t.Fatalf("version = %d", ir.Version)
	}
	if len(ir.Root) != 1 {
		t.Fatalf("root len = %d", len(ir.Root))
	}
	html, err := RenderHTML("../fixtures/hello.crepus", map[string]any{"name": "Ada"})
	if err != nil {
		t.Fatal(err)
	}
	if html != `<div data-crepus-kind="stack" data-axis="column">Hello Ada</div>` {
		t.Fatalf("html = %s", html)
	}
}

func TestViewSessionDispatch(t *testing.T) {
	os.Setenv("CREPUS_ALLOWED_DIR", "..")
	session := NewViewSession("../fixtures/interactive.crepus", map[string]any{"count": "1"})
	html, err := session.RenderHTML()
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(html, "Count 1") {
		t.Fatalf("html = %s", html)
	}
	ir, err := session.Dispatch(Event{Handler: "bind:count:2"})
	if err != nil {
		t.Fatal(err)
	}
	raw := fmt.Sprint(ir.Root)
	if !strings.Contains(raw, "Count 2") {
		t.Fatalf("ir = %s", raw)
	}
}

func TestRenderIRTraversalDenied(t *testing.T) {
	os.Setenv("CREPUS_ALLOWED_DIR", ".")
	_, err := RenderIR("../fixtures/hello.crepus", map[string]any{"name": "Ada"})
	if err == nil {
		t.Fatal("expected traversal denied error")
	}
	if !strings.Contains(err.Error(), "path traversal denied") {
		t.Fatalf("unexpected error: %v", err)
	}
}
