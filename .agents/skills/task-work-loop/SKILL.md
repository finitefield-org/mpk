---
name: task-work-loop
description: Implement exactly one requested project task, then run code-review and fix iterations until review produces no findings, then commit and push the completed work. Use when the user asks to implement one task or ticket and wants the agent to review, address findings, repeat until clean, and publish the result; Japanese triggers include「ひとつのタスクを実装」「レビューと修正を繰り返す」「指摘点がなくなるまで」「指摘点がなくなったらコミットとプッシュ」.
---

# Task Work Loop

Use this loop only when the user's request explicitly authorizes both commit and push; permission to implement a task alone is insufficient.

## Workflow

1. Identify the single task to implement from the user request and local project docs. If multiple tasks are requested, stop and ask which one to do first.
2. Read the repository guidance and task specification before editing. Prefer local docs, tickets, plans, and existing code over assumptions.
3. Implement only the selected task. Keep changes scoped to the task's deliverables and acceptance criteria.
4. Run the appropriate verification for the changed area. Prefer repository-standard commands when available, and include targeted tests for the task's behavior.
5. Review the full task diff without delegating to a review skill unless the user explicitly requests one. Use a code-review stance: findings first, ordered by severity, with file and line references.
6. If review finds issues, fix them, rerun the relevant verification, and review the new diff again.
7. Repeat fix and review until the latest review has no findings.
8. Once the latest review has no findings, stage only the task changes, create an intentional commit, and push the current branch. Do not force-push or change remotes unless the user explicitly requests it.
9. Report the final status with implemented scope, review result, verification commands, commit hash, pushed branch, and working tree state.

## Review Rules

- Treat batch-level or workflow-level contract mismatches as findings even when unit tests pass.
- Check that tests fail or would fail on the reviewed issue before trusting a fix.
- Do not count scheduler stops, infrastructure timeouts, or other external interruptions as deterministic implementation failures unless the specification says so.
- Do not broaden the task during rework. Defer adjacent improvements to later tasks unless they are required to fix a review finding.
- If a review finding cannot be fixed safely without changing task scope, explain the blocker and stop for user direction.

## Completion Bar

The loop is complete only when all are true:

- the selected task's documented deliverables are implemented,
- the latest review has no findings,
- the relevant verification has passed or any unrun command is explicitly reported with the reason,
- the clean-reviewed changes are committed and pushed,
- the working tree state is clear to the user.
