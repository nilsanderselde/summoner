// Summoner - Deterministic, Headless-First DAW
// Copyright (C) 2026 nilsanderselde
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

//! GitHub integration and Patch-to-PR exporter.

use std::env;

/// Create and checkout a new patch branch in the Git repository.
pub fn create_patch_branch<'a>(
    repo: &'a git2::Repository,
    branch_name: &str,
) -> Result<git2::Branch<'a>, String> {
    let head_commit = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;

    let branch = repo
        .branch(branch_name, &head_commit, true)
        .map_err(|e| e.to_string())?;

    let refname = format!("refs/heads/{}", branch_name);
    repo.set_head(&refname).map_err(|e| e.to_string())?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .map_err(|e| e.to_string())?;

    Ok(branch)
}

/// Push a local branch to a remote Git URL.
pub fn push_branch_to_remote(
    repo: &git2::Repository,
    branch_name: &str,
    remote_url: &str,
) -> Result<(), String> {
    let token = env::var("GITHUB_TOKEN").ok();
    let mut remote = repo
        .remote_anonymous(remote_url)
        .map_err(|e| format!("Failed to create remote: {}", e))?;

    let mut callbacks = git2::RemoteCallbacks::new();
    if let Some(tok) = token {
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext("x-access-token", &tok)
        });
    }

    let mut options = git2::PushOptions::new();
    options.remote_callbacks(callbacks);

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote
        .push(&[&refspec], Some(&mut options))
        .map_err(|e| format!("Git push failed: {}", e))?;

    Ok(())
}

/// Submit a Pull Request to GitHub REST API using `ureq`.
pub fn create_github_pr(
    token: &str,
    owner: &str,
    repo_name: &str,
    branch: &str,
    title: &str,
    body: &str,
) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo_name}/pulls");

    let payload = format!(
        "{{\"title\": \"{}\", \"head\": \"{}\", \"base\": \"master\", \"body\": \"{}\"}}",
        title.replace('"', "\\\""),
        branch.replace('"', "\\\""),
        body.replace('"', "\\\"")
    );

    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "Summoner-DAW")
        .set("Accept", "application/vnd.github.v3+json")
        .send_string(&payload)
        .map_err(|e| format!("GitHub API HTTP request failed: {}", e))?;


    let resp_str = resp
        .into_string()
        .map_err(|e| format!("Failed to read GitHub response: {}", e))?;

    Ok(resp_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_patch_branch() {
        let temp_dir = std::env::temp_dir().join(format!("summoner_branch_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let repo = summoner_project::git_dag::open_or_init_repo(&temp_dir).unwrap();
        let proj = summoner_project::create_default_project("Branch Test");
        summoner_project::git_dag::commit_project_state(&repo, &proj, "Initial").unwrap();

        let branch = create_patch_branch(&repo, "patch/test-branch").unwrap();
        assert_eq!(branch.name().unwrap().unwrap(), "patch/test-branch");

        let head = repo.head().unwrap();
        assert_eq!(head.shorthand().unwrap(), "patch/test-branch");

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
