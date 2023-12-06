# Crypto Ecosystem Explorer
Explore the latest commits and summaries of active or new projects in the crypto industry.

## Overview
The Crypto Ecosystem Explorer is a tool designed to gather insights into recent developments within the cryptocurrency landscape. By monitoring the latest commits in the Electric Capital's [Crypto Ecosystem](https://github.com/electric-capital/crypto-ecosystems) repository, the tool identifies changed data files and extracts URLs of associated repositories. Subsequently, it fetches the READMEs of these projects to compile a summary using a Language Model (NYI).

## Usage
### Installation
Clone the repository:

```shell
git clone https://github.com/GrandinLuc/ecosystems_scrapper.git
cd ecosystems_scrapper
```

Build:
```shell
cargo build
```

Get all the projects that had a change in the last commits:
```shell
cargo run --bin latest_repos
```

Explore the ReadMes of the projects saved in the last step:
```shell
cargo run --bin readme_fetcher
```

Summarization with Language Model
todo


Contributing
We welcome contributions to enhance the functionality and coverage of the Crypto Ecosystem Explorer. If you would like to contribute:

Fork the project.
Create a new branch (git checkout -b feature/your_feature).
Commit your changes (git commit -am 'Add your_feature').
Push to the branch (git push origin feature/your_feature).
Open a Pull Request.

Feel free to customize the sections, add more details, and include any relevant information. Providing clear instructions, usage examples, and a contributing guide can significantly improve the readability and usability of your README.