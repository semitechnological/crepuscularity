# Task for delegate

FIX TWO PR BRANCHES by checking out, editing, pushing:

1. PR #151 (branch: perf-embedded-tree-indexing-14497208190404845440)
   - File: crates/crepuscularity-embedded/src/document.rs.orig
   - Action: Remove this file from the branch (git rm, commit, push)
   
2. PR #163 (branch: refactor-run-all-suites-483691057680500632)
   - File: crates/crepuscularity-lite/src/bench_plugin.rs.orig
   - Action: Remove this file from the branch (git rm, commit, push)

CWD: /Users/undivisible/projects/crepuscularity

Workflow for each:
1. `git fetch origin`
2. `git checkout <branch>`
3. `git rm <orig-file>`
4. `git commit -m "chore: remove accidental .orig backup file"`
5. `git push origin <branch>`
6. `git checkout main`

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