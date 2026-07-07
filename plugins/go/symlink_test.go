package crepuscularity

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRenderIRSymlinkTraversalDenied(t *testing.T) {
	// Setup allowed dir
	allowedDir := t.TempDir()
	os.Setenv("CREPUS_ALLOWED_DIR", allowedDir)

	// Setup secret outside
	secretPath := filepath.Join(t.TempDir(), "secret.txt")
	os.WriteFile(secretPath, []byte("secret"), 0644)

	// Create symlink inside allowedDir
	linkPath := filepath.Join(allowedDir, "link.crepus")
	err := os.Symlink(secretPath, linkPath)
	if err != nil {
		t.Skip("symlinks not supported")
	}

	_, err = RenderIR(linkPath, map[string]any{})
	if err == nil {
		t.Fatal("expected traversal denied error for symlink, got nil")
	}
	if !strings.Contains(err.Error(), "path traversal denied") {
		t.Fatalf("unexpected error: %v", err)
	}
}
