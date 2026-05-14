package crepuscularity

import (
	"bytes"
	"encoding/json"
	"fmt"
	"html"
	"os"
	"os/exec"
	"strings"
)

type ViewIr struct {
	Version int              `json:"version"`
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

func RenderHTML(path string, context map[string]any) (string, error) {
	ir, err := RenderIR(path, context)
	if err != nil {
		return "", err
	}
	var out strings.Builder
	for _, node := range ir.Root {
		out.WriteString(renderNode(node))
	}
	return out.String(), nil
}

func renderNode(node map[string]any) string {
	switch node["kind"] {
	case "text":
		return html.EscapeString(fmt.Sprint(node["content"]))
	case "stack", "scroll":
		var out strings.Builder
		out.WriteString(`<div data-crepus-kind="`)
		out.WriteString(html.EscapeString(fmt.Sprint(node["kind"])))
		out.WriteString(`" data-axis="`)
		out.WriteString(html.EscapeString(fmt.Sprint(node["axis"])))
		out.WriteString(`">`)
		for _, child := range nodeList(node["children"]) {
			out.WriteString(renderNode(child))
		}
		out.WriteString("</div>")
		return out.String()
	case "button":
		label := html.EscapeString(fmt.Sprint(node["label"]))
		if node["onClick"] != nil {
			return `<button data-onclick="` + html.EscapeString(fmt.Sprint(node["onClick"])) + `">` + label + `</button>`
		}
		return "<button>" + label + "</button>"
	case "image":
		return `<img src="` + html.EscapeString(fmt.Sprint(node["src"])) + `" alt="` + html.EscapeString(fmt.Sprint(node["alt"])) + `">`
	case "slotRotate":
		phrases := valueList(node["phrases"])
		if len(phrases) == 0 {
			return `<span data-crepus-kind="slotRotate"></span>`
		}
		return `<span data-crepus-kind="slotRotate">` + html.EscapeString(fmt.Sprint(phrases[0])) + `</span>`
	case "input":
		bind := html.EscapeString(fmt.Sprint(node["bind"]))
		placeholder := html.EscapeString(fmt.Sprint(node["placeholder"]))
		if node["multiline"] == true {
			return `<textarea data-bind="` + bind + `" placeholder="` + placeholder + `"></textarea>`
		}
		return `<input data-bind="` + bind + `" placeholder="` + placeholder + `">`
	case "picker":
		var out strings.Builder
		out.WriteString(`<select data-bind="`)
		out.WriteString(html.EscapeString(fmt.Sprint(node["bind"])))
		out.WriteString(`">`)
		for _, option := range nodeList(node["options"]) {
			out.WriteString(`<option value="`)
			out.WriteString(html.EscapeString(fmt.Sprint(option["value"])))
			out.WriteString(`">`)
			out.WriteString(html.EscapeString(fmt.Sprint(option["label"])))
			out.WriteString(`</option>`)
		}
		out.WriteString(`</select>`)
		return out.String()
	default:
		return ""
	}
}

func nodeList(value any) []map[string]any {
	raw, ok := value.([]any)
	if !ok {
		return nil
	}
	out := make([]map[string]any, 0, len(raw))
	for _, item := range raw {
		if node, ok := item.(map[string]any); ok {
			out = append(out, node)
		}
	}
	return out
}

func valueList(value any) []any {
	raw, _ := value.([]any)
	return raw
}
