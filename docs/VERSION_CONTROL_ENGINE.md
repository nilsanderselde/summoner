# Summoner DAW — Embedded Version Control Engine

This document details the embedded **`libgit2` Micro-Commit Engine**, session DAG history, undo/redo mechanics, and patch-to-PR export features in `summoner_project`.

---

## 1. Overview & Philosophy

In traditional DAWs, undo history is an in-memory stack lost upon closing the project or crashing.

Summoner DAW embeds a native Git repository (`libgit2` via the `git2` crate) directly inside every project directory (`.summoner/`). Every parameter tweak, sequence edit, or node routing change creates an atomic, in-memory **micro-commit** on a Directed Acyclic Graph (DAG).

```
[Commit A: Initial Session]
        │
[Commit B: Add Track 1]
        │
[Commit C: Tweaked Cutoff to 2400 Hz] ◄── Undo / Redo Branch Traversal
        │
[Commit D: Added Microtonal 19-EDO]
```

---

## 2. Key Architecture Attributes

- **Atomic Micro-Commits:** Small, automated Git commits are generated behind every user interaction without interrupting real-time audio playback.
- **DAG Navigation for Undo/Redo:** Moving backward or forward through undo history is a Git commit traversal (`git checkout` / detached HEAD / branch pointer movement).
- **Non-Destructive History:** Edits are never permanently overwritten. Reverting an action creates a commit or traverses history, preserving full auditability.
- **Session Branching:** Users can branch session states to test alternative sound design directions, mix variations, or generative sequences.

---

## 3. Patch Export & Pull Request Workflow

Because session files are stored in Git-native format:

1. **Patch Generation (`summon export-patch`):** Export session branches or state diffs as standard Git `.patch` files.
2. **Patch to PR (`summon pr-export`):** Export sound design changes or preset creations directly as GitHub Pull Requests or Git patches for collaborative music production.
3. **Merging Sessions:** Merge tracks or presets from another contributor's session using Git merge algorithms.
