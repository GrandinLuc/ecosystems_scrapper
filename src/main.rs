// URL we scrap
// https://github.com/machinefi/Bike-Sharing-DePIN-Webinar
// URL we have to GET request
// https://raw.githubusercontent.com/machinefi/Bike-Sharing-DePIN-Webinar/main/README.md
use git2::{Repository, Oid, Diff};

use anyhow::{anyhow, Result};
use reqwest;

// GitHub repository information
const OWNER: &str = "electric-capital";
const REPO: &str  = "crypto-ecosystems";

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

fn get_diff(old_commit_hash: String, new_commit_hash: String) -> Vec<String>  {

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
    
    let deltas = diff.deltas().filter_map(|delta| {
        delta.new_file().path()
    }).filter_map(|path| {
        path.to_str()
    }).collect::<Vec<&str>>();


    deltas.into_iter().map(|x| x.to_owned()).collect::<Vec<String>>()
    
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

    let old_commit_hash = "f7663fca12311d886fbcd3009e6069b7a697490e".to_string();
    let new_commit_hash = "7acb5d17f258abbb1e7f1e9f2c58a1c2924cd41d".to_string();
    let diff_files = get_diff(old_commit_hash, new_commit_hash);


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
