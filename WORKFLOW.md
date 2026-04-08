Suggested workflow (Git-based, works in Forgejo/GitHub)[Based on GPT5-mini]

1. Create an issue
   - Title: short summary
   - Body: goals, acceptance criteria, mockups, steps to reproduce (if bug), labels, assignee, milestone.

2. Create a feature branch
   - Link branch to the issue: name convention: feature,bugfix,chore,docs,test,refactor,perf/<issue-number>-short-desc
   - Command:
     ```
     git checkout -b feature/123-add-login
     ```
   - Push branch and set upstream:
     ```
     git push -u origin feature/123-add-login
     ```

3. Work in small, focused commits
   - Make logical commits with clear messages.
   - In commits, reference the issue: `git commit -m "Fix #123: add login form and validation"`
   - Use the `Fixes #123` keyword to auto-close issue #123 on merge, use `Related #123` for linking but manual closing of the issue.

4. Open a pull/merge request (PR/MR)
   - Base: target branch (e.g., main or develop).
   - Title: include issue number (e.g., "Add login form (#123)").
   - Description: link the issue, summarize changes, list testing steps, reference related PRs/MRs.
   - Use reviewers, labels, milestone, and link the issue in the PR description (e.g., "Closes #123" to auto-close on merge, or "Related: #123" to keep issue open).

5. Continuous integration & automated checks
   - Ensure CI runs tests, linters, build checks on the PR.
   - Fix failures in the same branch/PR.

6. Request review and iterate
   - Respond to review comments with commits on the same branch.
   - Keep PR small and focused to ease review.

7. Merge with the correct strategy
   - Use merge strategy agreed on (squash, rebase & merge, or merge commit).
   - If using squash, ensure PR description contains a good summary because that becomes the commit message.

8. Close and verify
   - If not auto-closed, manually close the issue when merged and verify deployment/testing as needed.
   - Update milestone/project board: move card to Done.

9. Post-merge housekeeping
   - Delete the remote branch:
     ```
     git push origin --delete feature/123-add-login
     ```
   - Pull latest main into local and keep branches up to date.

Optional best-practice additions
- Draft PR while still working to signal WIP.
- Reference commits/PRs in the issue comments to keep history discoverable.
- Use project boards to move the issue card through workflow columns (Backlog → In Progress → Review → Done).
- Tag releases with version and milestone when shipping.

This workflow ensures traceability: issue ↔ branch ↔ commits ↔ PR/MR ↔ merge, and integrates milestones/projects for planning and tracking.