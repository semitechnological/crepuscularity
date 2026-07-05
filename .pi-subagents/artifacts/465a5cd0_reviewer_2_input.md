# Task for reviewer

[Read from: /Users/undivisible/projects/crepuscularity/plan.md, /Users/undivisible/projects/crepuscularity/progress.md]

REVIEW THESE REFACTOR PRs. For each, `gh pr diff <number>` then review if the change is a net simplification. Key metric: net lines + complexity reduction. Flag YAGNI, over-engineering.

PR #169: Simplify apply_borders_shadows matching
  crates/crepuscularity-runtime/src/styler.rs
PR #168: Refactor JNI string evaluation boilerplate into helpers
  crates/crepuscularity-cli/templates/native/rust/src/lib.rs
PR #167: Break apart native shell build pipeline
  crates/crepuscularity-cli/src/native.rs
PR #160: Refactor build_site_wasm for maintainability
  crates/crepuscularity-cli/src/web.rs
PR #158: Refactor draw_frame in benchmark_tui
  crates/crepuscularity-cli/src/benchmark_tui.rs
PR #157: Simplify render_seo_head fallback logic
  crates/crepuscularity-cli/src/web.rs
PR #156: Extract write_runtime_assets
  crates/crepuscularity-cli/src/webext.rs
PR #154: Simplify render_crepus_pages looping
  crates/crepuscularity-cli/src/webext.rs
PR #153: Extract setup from build_wasm_runtime
  crates/crepuscularity-cli/src/webext.rs
PR #163: Refactor run_all_suites to reduce complexity
  crates/crepuscularity-lite/src/bench_plugin.rs (+501 -82)
PR #150: Extract logic from from_manifest_for_browser
  crates/crepuscularity-webext/src/manifest.rs

Return table: PR# | Title | Net ± | Simpler? (✅/⚠️/❌) | Notes

CWD: /Users/undivisible/projects/crepuscularity

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

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