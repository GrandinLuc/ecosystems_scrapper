// We check all the files that changed between twos commits on the ecosystems repo (https://github.com/electric-capital/crypto-ecosystems)
// Then we parse the toml files that corresponds to projects that had changes between these two commits
// We save a json containing all the projects that changed and the github urls that corresponds

use std::collections::HashMap;
use std::fs::File;

use anyhow::Result;
use git2::{Oid, Repository};
use std::io::{BufWriter, Write};
use toml::Value;

// GitHub repository information
const OWNER: &str = "electric-capital";
const REPO: &str = "crypto-ecosystems";

/// URL we scrap
/// https://github.com/machinefi/Bike-Sharing-DePIN-Webinar
/// URL we have to GET request
/// https://raw.githubusercontent.com/machinefi/Bike-Sharing-DePIN-Webinar/main/README.md
fn repo_url_to_readme_url(repo_url: &str) -> Vec<String> {
    let last_part = repo_url
        .strip_prefix("https://github")
        .expect("Failed to remove the start of the url");

    vec![
        String::from("https://raw.githubusercontent") + last_part + "/main/README.md",
        String::from("https://raw.githubusercontent") + last_part + "/master/README.md",
    ]
}

fn get_changed_files_raw(old_commit_hash: String, new_commit_hash: String) -> Result<Vec<String>> {
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
    let diff =
        match repo.diff_tree_to_tree(Some(&old_commit.tree()?), Some(&new_commit.tree()?), None) {
            Ok(diff) => diff,
            Err(e) => panic!("failed to get diff: {}", e),
        };

    let changed_files: Vec<Vec<u8>> = diff
        .deltas()
        .filter_map(|delta| delta.new_file().path())
        .filter_map(|path| {
            if path.extension()? != "toml" {
                return None;
            }
            if !path.starts_with("data/ecosystems") {
                return None;
            }
            let binding = match new_commit
                .tree()
                .expect("Couldn't get the tree of the new commit")
                .get_path(path)
            {
                Ok(value) => value
                    .to_object(&repo)
                    .expect("Couldn't convert tree to object")
                    .into_blob()
                    .expect("Couldn't convert object to blob"),
                Err(_) => return None,
            };

            let content = binding.content();
            Some(content.to_owned())
        })
        .collect();

    Ok(changed_files
        .into_iter()
        .map(|x| String::from_utf8_lossy(&x).to_string())
        .collect())
}

fn extract_urls(toml_files: Vec<Value>) -> HashMap<String, Vec<String>> {
    toml_files
        .into_iter()
        .filter_map(|toml_file| {
            let title = toml_file["title"]
                .as_str()
                .expect("Missing or invalid title")
                .to_string();

            let mut repos: Vec<String> = vec![];
            match toml_file.get("repo").and_then(Value::as_array) {
                Some(value) => {
                    for repo in value {
                        if let Some(url) = repo.get("url").and_then(Value::as_str) {
                            if url.contains("github.com") {
                                repos.push(url.to_string())
                            }
                        }
                    }
                }
                None => return None,
            }
            Some((title, repos))
        })
        .collect()
}

fn get_readme_urls(repo_urls: Vec<String>) -> Vec<String> {
    repo_urls
        .into_iter()
        .map(|repo_url| repo_url_to_readme_url(&repo_url))
        .flat_map(|readme_pair| readme_pair.into_iter())
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let old_commit_hash = "e5935b7c2249ff75851e2d31f79a59791e61d753".to_string();
    let new_commit_hash = "cd4d6d144e66bd8092433818de0d0f7780c4dfd5".to_string();
    let changed_files = get_changed_files_raw(old_commit_hash, new_commit_hash);

    let parsed_tomls: Vec<Value> = changed_files?
        .into_iter()
        .map(|file| toml::from_str(&file).expect("Failed to parse TOML"))
        .collect();

    let extracted_urls_by_project = extract_urls(parsed_tomls);

    let readmes_by_project: HashMap<String, Vec<String>> = extracted_urls_by_project
        .into_iter()
        .map(|(project_name, project_urls)| (project_name, get_readme_urls(project_urls)))
        .collect();

    let file = File::create("dump.json")?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &readmes_by_project)?;
    writer.flush()?;

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
