# Task for delegate

MERGE ALL CLEAN PRs using `gh pr merge --squash --delete-branch`.

These PRs are verified clean (no issues found in review):
Security:
- 176 (fix/python-crepus-bin-vuln)
- 174 (fix/ruby-crepus-bin-security)
- 172 (fix-python-plugin-path-traversal)
- 162 (fix-ruby-plugin-path-traversal)
- 177 (security-go-plugin-path-traversal)
- 175 (fix-php-lfi)

Tests:
- 170 (test-dom-attr-error-paths)
- 166 (add-error-path-tests-for-eval-guest-from-config-file)
- 165 (test-improve-template-error-paths)
- 164 (test-draw-full-error-path)
- 155 (test-write-ppm-error)
- 148 (test-template-from-path-error)
- 171 (test-clipboard-error-paths)

Refactor:
- 169 (refactor-borders-shadows)
- 168 (refactor/jni-evaluation-helpers)
- 167 (break-apart-native-shell-pipeline)
- 160 (refactor-build-site-wasm)
- 158 (refactor-benchmark-tui)
- 156 (refactor-build-extension-inner)
- 154 (simplify-render-crepus-pages-looping)
- 153 (refactor-build-wasm-runtime)
- 150 (fix/manifest-refactor)

Perf:
- 149 (optimize-manifest-css-allocation)
- 159 (perf-reactive-hashset)

Do NOT merge: 151, 152, 157, 161, 163, 173 (these need fixes first).

For each: `gh pr merge <number> --squash --delete-branch`

Handle merge failures gracefully - if a PR can't merge (conflicts), skip it and report.

CWD: /Users/undivisible/projects/crepuscularity

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: required by reviewer.

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```