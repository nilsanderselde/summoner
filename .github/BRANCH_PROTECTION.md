# Master Branch Protection Rules

The `master` branch of **nilsanderselde/summoner** is protected by GitHub Branch Protection rules to ensure code stability, security, real-time audio safety, and clean linear history.

---

## 🔒 Active Protection Rules

### 1. Required Status Checks (CI Gates)
All of the following GitHub Actions CI jobs must pass cleanly before any code can be merged into `master`:
- **`Build & Test (ubuntu-latest)`**
- **`Build & Test (windows-latest)`**
- **`Build & Test (macos-latest)`**
- **`CodeQL Security Analysis`**
- **`Benchmark Regression Guard`**
- **`Fuzz Target Harnesses`**
- **`Security Audit`**

- **Strict Up-to-Date Requirement (`strict: true`):** Pull Request branches must be up-to-date with `master` before merging.

---

### 2. Pull Request Review Requirements
- **Approving Reviews:** At least **1 approving review** is required for all PRs.
- **Stale Review Dismissal:** Approvals are automatically dismissed when new commits are pushed to the PR branch (`dismiss_stale_reviews: true`).
- **Last Push Approval:** Approvals must be made on the latest commit hash (`require_last_push_approval: true`).
- **Conversation Resolution:** All review comment threads must be resolved before merging (`required_conversation_resolution: true`).

---

### 3. History & Push Protections
- **Linear History Enforced (`required_linear_history: true`):** Require rebased or squashed commits to keep the Git DAG clean.
- **Force Pushes Blocked (`allow_force_pushes: false`):** Prevents accidental `--force` pushes from overwriting history on `master`.
- **Branch Deletions Blocked (`allow_deletions: false`):** Prevents deletion of the `master` branch.
