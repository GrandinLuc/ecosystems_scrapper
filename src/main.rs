use serde::Deserialize;
use std::borrow::BorrowMut;
// URL we scrap
// https://github.com/machinefi/Bike-Sharing-DePIN-Webinar
// URL we have to GET request
// https://raw.githubusercontent.com/machinefi/Bike-Sharing-DePIN-Webinar/main/README.md
use git2::{Diff, Oid, Repository};
use toml::Value;

use anyhow::{anyhow, Result};
use reqwest;

// GitHub repository information
const OWNER: &str = "electric-capital";
const REPO: &str = "crypto-ecosystems";

fn repo_url_to_readme_url(repo_url: &str) -> Vec<String> {
    let last_part = repo_url
        .strip_prefix("https://github")
        .expect("Failed to remove the start of the url");

    vec![
        String::from("https://raw.githubusercontent") + last_part + "/main/README.md",
        String::from("https://raw.githubusercontent") + last_part + "/master/README.md",
    ]
}

async fn fetch_readme(url: &str) -> Result<String> {
    // Make the GET request
    let response = reqwest::get(url).await?;

    // Check if the request was successful (status code 2xx)
    if response.status().is_success() {
        // Read the response body as a string
        return Ok(response.text().await?);
    } else {
        return Err(anyhow!(response.status()));
    }
}

fn get_changed_files_raw(old_commit_hash: String, new_commit_hash: String) -> Vec<String> {
    // Create a temporary directory for the cloned repository
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");

    // Clone the GitHub repository
    let repo_url = format!("https://github.com/{}/{}.git", OWNER, REPO);
    let repo = match Repository::clone(repo_url.as_str(), temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone repository: {}", e),
    };

    // Resolve commit OIDs
    let old_oid = match Oid::from_str(&old_commit_hash) {
        Ok(oid) => oid,
        Err(e) => panic!("failed to parse old commit OID: {}", e),
    };

    let new_oid = match Oid::from_str(&new_commit_hash) {
        Ok(oid) => oid,
        Err(e) => panic!("failed to parse new commit OID: {}", e),
    };

    // Get old and new commits
    let old_commit = match repo.find_commit(old_oid) {
        Ok(commit) => commit,
        Err(e) => panic!("failed to find old commit: {}", e),
    };

    let new_commit = match repo.find_commit(new_oid) {
        Ok(commit) => commit,
        Err(e) => panic!("failed to find new commit: {}", e),
    };

    // Get the diff between the two commits
    let diff = match repo.diff_tree_to_tree(
        Some(&old_commit.tree().unwrap()),
        Some(&new_commit.tree().unwrap()),
        None,
    ) {
        Ok(diff) => diff,
        Err(e) => panic!("failed to get diff: {}", e),
    };

    let changed_files: Vec<Vec<u8>> = diff
        .deltas()
        .filter_map(|delta| delta.new_file().path())
        .filter_map(|path| {
            if path.extension().unwrap() != "toml" {
                return None;
            }
            if !path.starts_with("data/ecosystems") {
                return None;
            }
            let binding = match new_commit.tree().unwrap().get_path(path) {
                Ok(value) => value.to_object(&repo).unwrap().into_blob().unwrap(),
                Err(_) => return None,
            };

            let content = binding.content();
            Some(content.to_owned())
        })
        .collect();

    changed_files
        .into_iter()
        .map(|x| String::from_utf8_lossy(&x).to_string())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // // Specify the URL you want to make a GET request to
    // let url = "https://github.com/machinefi/Bike-Sharing-DePIN-Webinar";

    // // Make the GET request
    // for e in repo_url_to_readme_url(url) {
    //     let response = fetch_readme(&e).await.unwrap();

    //     println!("Response: {:?}", response);
    // }

    let old_commit_hash = "e5935b7c2249ff75851e2d31f79a59791e61d753".to_string();
    let new_commit_hash = "cd4d6d144e66bd8092433818de0d0f7780c4dfd5".to_string();
    let changed_files = get_changed_files_raw(old_commit_hash, new_commit_hash);

    let parsed_toml: Value = toml::from_str(&changed_files[0]).expect("Failed to parse TOML");

    println!(
        "file: {}",
        parsed_toml["title"]
            .as_str()
            .expect("Missing or invalid title")
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn can_convert_repo_url_to_readme_url() -> Result<()> {
        let url = "https://github.com/machinefi/Bike-Sharing-DePIN-Webinar";

        assert_eq!(repo_url_to_readme_url(url), vec!["https://raw.githubusercontent.com/machinefi/Bike-Sharing-DePIN-Webinar/main/README.md", "https://raw.githubusercontent.com/machinefi/Bike-Sharing-DePIN-Webinar/master/README.md"]);

        Ok(())
    }
}
