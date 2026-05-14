package crepuscularity

import (
	"fmt"
	"strings"
	"testing"
)

func TestRenderIR(t *testing.T) {
	ir, err := RenderIR("../fixtures/hello.crepus", map[string]any{"name": "Ada"})
	if err != nil {
		t.Fatal(err)
	}
	if ir.Version != 3 {
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
