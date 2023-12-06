use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ReadmesByProject(HashMap<String, Vec<String>>);

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

fn read_line(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut file = File::open("dump.json")?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer)?;

    let readme_urls_by_project: ReadmesByProject =
        serde_json::from_str(&buffer).expect("Failed to deserialize");

    let project_name = read_line("What project do you want to get the Readmes from ?");
    let project = readme_urls_by_project.0.get(&project_name).expect("Couldn't find this project");
    
    let mut url_count = 0;
    loop {
        if url_count < project.len() {
            let current_url = &project[url_count];
            match fetch_readme(current_url).await {
                Ok(readme_content) => {
                    if read_line("Do you want to read the next ReadMe for this project ? (yes/no)") != String::from("yes") {
                        break;
                    }
                    println!("For the repo {} we have the following readme", current_url);
                    println!("{}\n", readme_content);

                }
                Err(err) => {
                    println!("No readme for repo: {:?}, with error: {:?}", current_url, err);
                }
            }
            
        }
        else {
            println!("All the readmes for this project have been read");
            break;
        }


        url_count+= 1;
    }
    
    Ok(())
}
