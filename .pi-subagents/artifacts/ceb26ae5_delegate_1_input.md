# Task for delegate

FIX TWO PR BRANCHES by checking out, editing, pushing:

1. PR #157 (branch: health-improve-render-seo-head-16646888490173264756)
   File: crates/crepuscularity-cli/src/web.rs
   Issue: `format_seo_tags` takes 6 individual params. Should take `&SiteHead` instead.
   
   Current caller (in render_seo_head):
   ```rust
   let seo = &head.seo;
   let title = head.title.as_deref().unwrap_or(&head.name);
   let description = head.description.as_deref().unwrap_or("");
   let og_type = head.og_type.as_deref().unwrap_or("website");
   let image = head.image.as_ref();
   let twitter_card = head.twitter_card.as_deref().unwrap_or("summary");
   format_seo_tags(seo, title, description, og_type, image, twitter_card)
   ```
   
   Fix: Change `format_seo_tags` to take `head: &SiteHead` and derive the values internally.
   Same behavior, fewer params. Use `gh pr diff 157` to see the current diff.

2. PR #173 (branch: fix-crepus-bin-command-injection-11422455829256234502)
   File: plugins/php/CrepuscularityPlugin.php
   Issue: Current crepusBin() has over-engineered 3-regex logic. Replace with simple separator check.
   
   Replace the crepusBin() function body starting from "$isBinaryName" through the 3 checks with:
   ```php
   if (preg_match('#[/\\\\]#', $bin)) {
       throw new RuntimeException('CREPUS_BIN must be a binary name, not a path');
   }
   return $bin;
   ```
   Keep the first few lines (empty check + control char check).
   
   Use `gh pr diff 173` to see current state.

CWD: /Users/undivisible/projects/crepuscularity

Workflow for each:
1. `git fetch origin`
2. `git checkout <branch>`
3. Edit the file
4. `git add -A && git commit -m "fix: simplify per review"`
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