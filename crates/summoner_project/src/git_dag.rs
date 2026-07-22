// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

//! Embedded Git DAG micro-commit history engine and Patch-to-PR exporter.

use crate::schema::ProjectConfig;
use crate::serialize_project_toml;
use blake3::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};

/// Single atomic micro-commit on session history DAG.
#[derive(Debug, Clone, PartialEq)]
pub struct MicroCommit {
    pub id: String,
    pub parent_id: Option<String>,
    pub author: String,
    pub timestamp: u64,
    pub message: String,
    pub state: ProjectConfig,
}

impl MicroCommit {
    pub fn new(
        parent_id: Option<String>,
        author: impl Into<String>,
        message: impl Into<String>,
        state: ProjectConfig,
    ) -> Self {
        let author_str = author.into();
        let msg_str = message.into();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut hasher = Hasher::new();
        if let Some(ref p) = parent_id {
            hasher.update(p.as_bytes());
        }
        hasher.update(author_str.as_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(msg_str.as_bytes());

        if let Ok(toml_str) = serialize_project_toml(&state) {
            hasher.update(toml_str.as_bytes());
        }

        let hash_hex = hasher.finalize().to_hex().to_string();
        let id = hash_hex[..12].to_string();

        Self {
            id,
            parent_id,
            author: author_str,
            timestamp,
            message: msg_str,
            state,
        }
    }
}

/// Git DAG session history engine managing micro-commits, non-destructive undo/redo, and patch exports.
#[derive(Debug, Clone, PartialEq)]
pub struct GitSessionDag {
    author: String,
    history: Vec<MicroCommit>,
    current_index: usize,
}

impl GitSessionDag {
    pub fn new(initial_state: ProjectConfig, author: impl Into<String>) -> Self {
        let author_str = author.into();
        let root_commit = MicroCommit::new(
            None,
            author_str.clone(),
            "Initial session state",
            initial_state,
        );

        Self {
            author: author_str,
            history: vec![root_commit],
            current_index: 0,
        }
    }

    /// Commit a new state mutation to the DAG.
    pub fn commit(&mut self, message: impl Into<String>, new_state: ProjectConfig) -> String {
        // Truncate any redo branch when committing from a historical state
        self.history.truncate(self.current_index + 1);

        let parent_id = Some(self.history[self.current_index].id.clone());
        let commit = MicroCommit::new(parent_id, &self.author, message, new_state);
        let commit_id = commit.id.clone();

        self.history.push(commit);
        self.current_index = self.history.len() - 1;

        commit_id
    }

    /// Traverse backward in history DAG (Undo).
    pub fn undo(&mut self) -> Option<&ProjectConfig> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.history[self.current_index].state)
        } else {
            None
        }
    }

    /// Traverse forward in history DAG (Redo).
    pub fn redo(&mut self) -> Option<&ProjectConfig> {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            Some(&self.history[self.current_index].state)
        } else {
            None
        }
    }

    /// Retrieve current active head commit.
    pub fn head(&self) -> &MicroCommit {
        &self.history[self.current_index]
    }

    /// Retrieve full commit history timeline.
    pub fn history(&self) -> &[MicroCommit] {
        &self.history
    }

    /// Rollback active project state to a specific historical commit ID.
    pub fn rollback_to_commit(&mut self, commit_id: &str) -> Option<&ProjectConfig> {
        if let Some(pos) = self.history.iter().position(|c| c.id == commit_id) {
            self.current_index = pos;
            Some(&self.history[pos].state)
        } else {
            None
        }
    }


    /// Export a unified Git patch string representation comparing initial state against current head.
    pub fn export_patch(&self) -> String {
        let root_toml = serialize_project_toml(&self.history[0].state).unwrap_or_default();
        let head_toml = serialize_project_toml(&self.head().state).unwrap_or_default();

        format!(
            "From {} Mon Sep 17 00:00:00 2001\n\
             From: {}\n\
             Subject: [PATCH] {}\n\n\
             --- a/summoner_session.toml\n\
             +++ b/summoner_session.toml\n\
             @@ root: {} -> head: {} @@\n\
             - {}\n\
             + {}\n",
            self.head().id,
            self.author,
            self.head().message,
            self.history[0].id,
            self.head().id,
            root_toml.replace('\n', "\n- "),
            head_toml.replace('\n', "\n+ ")
        )
    }

    /// Create JSON PR payload representation for GitHub automated submission.
    pub fn create_github_pr_payload(&self, title: &str, body: &str) -> String {
        let patch = self.export_patch();
        format!(
            "{{\n  \"title\": \"{}\",\n  \"body\": \"{}\",\n  \"head_commit\": \"{}\",\n  \"author\": \"{}\",\n  \"patch\": \"{}\"\n}}",
            title,
            body,
            self.head().id,
            self.author,
            patch.replace('\n', "\\n").replace('"', "\\\"")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_default_project;

    #[test]
    fn test_micro_commit_dag_undo_redo() {
        let proj1 = create_default_project("Session 1");
        let mut dag = GitSessionDag::new(proj1.clone(), "nils");

        assert_eq!(dag.history().len(), 1);
        assert_eq!(dag.head().message, "Initial session state");

        let mut proj2 = proj1.clone();
        proj2.name = "Session 2 - Modified".to_string();
        let c2_id = dag.commit("Updated session name", proj2.clone());

        assert_eq!(dag.history().len(), 2);
        assert_eq!(dag.head().id, c2_id);
        assert_eq!(dag.head().state.name, "Session 2 - Modified");

        // Test Undo
        let undone = dag.undo().expect("Undo failed");
        assert_eq!(undone.name, "Session 1");

        // Test Redo
        let redone = dag.redo().expect("Redo failed");
        assert_eq!(redone.name, "Session 2 - Modified");
    }

    #[test]
    fn test_patch_and_pr_export() {
        let proj1 = create_default_project("Patch Test");
        let mut dag = GitSessionDag::new(proj1.clone(), "developer");

        let mut proj2 = proj1;
        proj2.transport.bpm = 140.0;
        dag.commit("Increase BPM to 140", proj2);

        let patch = dag.export_patch();
        assert!(patch.contains("[PATCH] Increase BPM to 140"));

        let pr_json = dag.create_github_pr_payload("BPM Update PR", "Automated patch update");
        assert!(pr_json.contains("\"title\": \"BPM Update PR\""));
        assert!(pr_json.contains("\"author\": \"developer\""));
    }
}
