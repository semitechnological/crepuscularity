package crepuscularity

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
)

type ViewIr struct {
	Version int             `json:"version"`
	Root    []map[string]any `json:"root"`
}

func crepusBin() string {
	if bin := os.Getenv("CREPUS_BIN"); bin != "" {
		return bin
	}
	return "crepus"
}

func RenderIR(path string, context map[string]any) (ViewIr, error) {
	payload, err := json.Marshal(map[string]any{
		"template": mustRead(path),
		"context":  context,
	})
	if err != nil {
		return ViewIr{}, err
	}
	cmd := exec.Command(crepusBin(), "native", "ir", "--stdin-json")
	cmd.Stdin = bytes.NewReader(payload)
	out, err := cmd.CombinedOutput()
	if err != nil {
		return ViewIr{}, fmt.Errorf("%w: %s", err, string(out))
	}
	var ir ViewIr
	if err := json.Unmarshal(out, &ir); err != nil {
		return ViewIr{}, err
	}
	return ir, nil
}

func mustRead(path string) string {
	data, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	return string(data)
}
