//! Shared cached-repository synchronization with checked exit status and token-redacted errors.

use std::{env, fs, path::Path};

use anyhow::{Context as _, Result};
use async_process::Command;

const GITHUB_DATA_PAT_ENV: &str = "GITHUB_DATA_PAT";

/// Sync a cached repository and report whether downstream import work should rerun.
/// Create the cache root, clone missing repositories, or replace caches whose origin no
/// longer matches. Git failures propagate with credentials redacted from diagnostics;
/// callers must pass a disposable data-cache directory, not a development checkout.
pub async fn sync_data_repo(data_root: &str, repo_url: &str, data_dir: &str) -> Result<bool> {
    fs::create_dir_all(data_root)?;

    let data_path = Path::new(data_root).join(data_dir);
    let remote_url = remote_url(repo_url);
    let mut changed = false;

    if data_path.exists() {
        match git_output(&["remote", "get-url", "origin"], &data_path).await {
            Ok(output) if strip_auth(output.trim()) == repo_url => {}
            _ => {
                // Existing servers may have older upstream clones cached under the same data path.
                fs::remove_dir_all(&data_path)?;
                changed = true;
            }
        }
    }

    if !data_path.exists() {
        git_output(
            &["clone", "--depth", "1", &remote_url, data_dir],
            Path::new(data_root),
        )
        .await?;
        changed = true;
    } else {
        // Store the PAT-backed remote when available so manual `git pull` works in the cache.
        git_output(&["remote", "set-url", "origin", &remote_url], &data_path).await?;
    }

    let output = git_output(&["pull"], &data_path).await?;
    if !output.contains("Already up to date.") {
        changed = true;
    }

    Ok(changed)
}

/// Run git in the cache directory and reject nonzero status with credential-redacted diagnostics.
async fn git_output(args: &[&str], current_dir: &Path) -> Result<String> {
    let output = git_command()
        .args(args)
        .current_dir(current_dir)
        .output()
        .await
        .with_context(|| {
            format!(
                "failed to start git {} in {}",
                redact(args.join(" ")),
                current_dir.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = [stderr.trim(), stdout.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let details = redact(details);
        let args = redact(args.join(" "));

        if details.is_empty() {
            anyhow::bail!("git {} failed with {}", args, output.status);
        }

        anyhow::bail!("git {} failed with {}: {}", args, output.status, details);
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Construct the git subprocess used for cache synchronization.
fn git_command() -> Command {
    Command::new("git")
}

/// Add configured GitHub data credentials to HTTPS URLs; leave local repository paths unchanged.
fn remote_url(repo_url: &str) -> String {
    if let Ok(token) = env::var(GITHUB_DATA_PAT_ENV) {
        let token = token.trim();

        if !token.is_empty() {
            return repo_url.replacen("https://", &format!("https://x-access-token:{token}@"), 1);
        }
    }

    repo_url.to_string()
}

/// Remove HTTPS user information before comparing cached and configured repository identities.
fn strip_auth(repo_url: &str) -> String {
    let Some(rest) = repo_url.strip_prefix("https://") else {
        return repo_url.to_string();
    };

    match rest.split_once('@') {
        Some((_, host_and_path)) => format!("https://{host_and_path}"),
        None => repo_url.to_string(),
    }
}

/// Remove the configured data token from strings before surfacing command diagnostics.
fn redact(value: String) -> String {
    let Ok(token) = env::var(GITHUB_DATA_PAT_ENV) else {
        return value;
    };
    let token = token.trim();

    if token.is_empty() {
        value
    } else {
        value.replace(token, "[redacted]")
    }
}
