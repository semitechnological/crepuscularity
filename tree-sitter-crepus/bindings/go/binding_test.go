package tree_sitter_crepus_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-crepus"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_crepus.Language())
	if language == nil {
		t.Errorf("Error loading Crepus grammar")
	}
}
