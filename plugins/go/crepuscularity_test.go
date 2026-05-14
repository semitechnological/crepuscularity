package crepuscularity

import "testing"

func TestRenderIR(t *testing.T) {
	ir, err := RenderIR("../fixtures/hello.crepus", map[string]any{"name": "Ada"})
	if err != nil {
		t.Fatal(err)
	}
	if ir.Version != 2 {
		t.Fatalf("version = %d", ir.Version)
	}
	if len(ir.Root) != 1 {
		t.Fatalf("root len = %d", len(ir.Root))
	}
}
