//! To get your repositories on Git in Rust CLI, you can use the Git command-line interface (CLI) tool called `git`. Here are the steps to follow:
//!
//! 1. Open your terminal or command prompt and navigate to the directory where you want to clone the repository.
//! 2. Clone the repository by running the command `git clone <repository URL>` where the repository URL is the URL of the repository you want to clone.
//! 3. Once the repository is cloned, navigate to the cloned repository directory by running the command `cd <repository name>`.
//! 4. To edit the privacy settings of your repository, you can use the Git API or the web interface provided by your Git hosting provider (such as GitHub, GitLab, or Bitbucket). Here's an example of how to make a repository private using the GitHub API:
//!    - First, you'll need to generate a personal access token (PAT) on GitHub by going to your account settings and selecting "Developer settings" > "Personal access tokens" > "Generate new token". Make sure to give the token the necessary permissions to modify repositories.
//!    - Next, run the following command to make the repository private:
//!
//!      ```
//!      curl -H "Authorization: token <your PAT>" -X PATCH https://api.github.com/repos/<your username>/<your repository name> -d '{"private": true}'
//!      ```
//!
//!      Replace `<your PAT>` with your personal access token, `<your username>` with your GitHub username, and `<your repository name>` with the name of your repository.
//!    - If you want to make multiple repositories private or public, you can create a script that loops through a list of repositories and makes the necessary changes using the Git API or the web interface provided by your Git hosting provider.
//!    - Alternatively, you can use a third-party tool like the GitHub CLI (`gh`) to manage your repositories from the command line. `gh` provides an easy-to-use interface for managing repositories, including creating, cloning, and modifying them.

#![deny(missing_docs)]

#[cfg(test)]
mod tests;

use anyhow::anyhow;
use github::Repo;
use serde::Deserialize;

pub(crate) type Result<T> = anyhow::Result<T, anyhow::Error>;

// Unicode symbols for success and error messages.
pub(crate) static SUCCESS_ICON: &str = "\u{2705}"; // ✅ green_check_unicode.
pub(crate) static ERROR_ICON: &str = "\u{274C}"; // ❌ red_x_unicode.

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    message: String,
    documentation_url: String,
}

/// This function makes a PATCH request to the GitHub API to update the privacy settings of a repository.
///
/// To make a public repository private using a personal access token (PAT) on GitHub, you need to have the `repo` scope in your PAT.
/// The `repo` scope allows the PAT to access and modify the repository, including changing its privacy settings.
///
/// Here are the steps to generate a PAT with the required scope:
///
/// 1. Go to your GitHub account settings and select "Developer settings" > "Personal access tokens" > "Generate new token".
/// 2. Give the token a name and select the `repo` scope.
/// 3. Click on "Generate token" to create the PAT.
///
/// Once you have generated the PAT, you can use it to make a public repository private by sending a
/// PATCH request to the GitHub API with the following payload:
///
/// ```
/// { "private": true }
/// ```
///
/// Make sure to include your PAT in the `Authorization` header of the request using the following format:
///
/// ```
/// Authorization: token <your PAT>
/// ```
///
/// Replace `<your PAT>` with your actual PAT value.
///
/// See also: [update-a-repository] https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#update-a-repository
#[tokio::main]
async fn main() -> Result<()> {
    // Load environment vairables from .env file.
    dotenv::dotenv().ok();

    // Prompt the user to enter the username and repository name.
    let username = prompter::prompt_user_input("Enter username: ")?;
    if username.is_empty() {
        return Err(anyhow!("{ERROR_ICON} `username` is required",));
    }

    // Get personal access token.
    let pat_token = std::env::var("PAT_TOKEN")
        .map(|token| match token.is_empty() {
            true => prompter::prompt_for_token().unwrap(),
            false => token,
        })
        .unwrap_or_else(|_| prompter::prompt_for_token().unwrap());
    if pat_token.is_empty() {
        return Err(anyhow!(
            "{ERROR_ICON} `PAT (Personal Access Token)` is required",
        ));
    }

    // let mut multiple_repository = Vec::new();
    let mut repositories: Vec<Repo>;

    // Prompt the user to select option for multiple repositories actions.
    let should_select_multiple_repos: bool = loop {
        let input =
            prompter::prompt_user_input("Do you want to modify multiple repositories?: (y/N) ")
                .unwrap_or_else(|_| "n".to_owned())
                .to_lowercase();
        if input == "y" || input == "n" {
            break input == "y";
        } else {
            println!("{ERROR_ICON} Please enter either `y` or `n` or `Ctrl/Cmd-C to quit`")
        }
    };

    // If user selects multiple repositories option.
    if should_select_multiple_repos {
        repositories = github::get_repos_request(&username.clone(), &pat_token).await?;
        let repos_ids: Vec<usize> =
            prompt_dialoguer::run_dialoguer(username.clone(), repositories.clone())?;
        if repos_ids.is_empty() {
            return Err(anyhow!(
                "{ERROR_ICON} No repositories were selected. Hint! Use <space> to select, then <Enter> to confirm.\nExiting",
            ));
        }
        repositories = repos_ids
            .into_iter()
            .map(|id| {
                let mut rep = repositories[id].clone();
                if rep.url.starts_with("https://api.github.com/repos") {
                    rep.url = rep.url.split("api.").collect::<Vec<_>>().join("");
                }
                rep
            })
            .collect();
    } else {
        let single_repository = prompter::prompt_user_input("Enter repository: ")?;
        if single_repository.is_empty() {
            return Err(anyhow!("{ERROR_ICON} `repository` is required",));
        }
        repositories = vec![Repo {
            name: single_repository.clone(),
            url: format!(
                "https://github.com/{username}/{repo}",
                username = username,
                repo = single_repository
            ),
            private: None, // FIXME: Can't know for sure if we should set this manually.
        }];
        // dbg!(&repositories);
    }

    for repo in repositories {
        // Construct the Authorization header and API URL.
        let api_url = format!(
            r#"https://api.github.com/repos/{username}/{repo}"#,
            username = username,
            repo = repo.name,
        );

        let leftpad = 30;
        let info_repo_url = style_repo_leftpad_url(&repo, Some(leftpad))?;

        // Prompt the user to enter the privacy setting for the repository.
        let privacy = 'l: loop {
            println!("{}", info_repo_url);
            let input = prompter::prompt_user_input(&format!(
                "  >> Make this repo private?: (true/false) ",
            ))
            .unwrap_or_else(|_| "false".to_owned());
            match input == "true" || input == "false" {
                true => break 'l input,
                false => println!("{ERROR_ICON} Please enter either `true` or `false`"),
            }
        };

        // FIXME: If repository is a public fork, and when attempted to make private,
        // this will panic and crash the program.
        github::post_request(repo.name, privacy, api_url, pat_token.clone()).await?;
    }

    Ok(())
}

pub(crate) fn style_repo_leftpad_url(repo: &Repo, leftpad: Option<usize>) -> Result<String> {
    use console::{measure_text_width, style};

    let pad = leftpad.unwrap_or(8);
    let name = style(&repo.name).yellow();
    let url = style(&repo.url).green().underlined();
    let name_width = measure_text_width(&repo.name);
    let result_leftpad = format!(
        "{name}{padding}{url}",
        name = name,
        padding = " ".repeat(pad.saturating_sub(name_width)),
        url = url
    );

    Ok(result_leftpad)
}

mod prompt_dialoguer {
    use super::Result;
    use crate::github::Repo;
    use console::style;
    use dialoguer::{theme::ColorfulTheme, MultiSelect};

    /// Enables user interaction and returns the result.
    ///
    /// The user can select the items with the 'Space' bar and on 'Enter' the indices of selected items will be returned.
    /// The dialog is rendered on stderr.
    /// Result contains `Vec<index>` if user hit 'Enter'.
    ///
    /// In this implementation, we use the `Url` crate to construct the URLs, `console` to style the
    /// URLs with underline, and `fmt::Write` to format the items with the repository name and
    /// clickable URL.
    pub(crate) fn run_dialoguer(_username: String, repos: Vec<Repo>) -> Result<Vec<usize>> {
        let parse_visibility = |is_private: bool| match is_private {
            false => style("public".to_string()).dim(),
            true => style("private".to_string()).yellow(),
        };

        let mut options: Vec<String> = Vec::new();
        for repo in &repos {
            options.push(format!(
                "{name} {visibility}",
                name = repo.name,
                visibility = parse_visibility(repo.private.unwrap())
            ));
        }

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Please select an option: (space to select, enter to confirm)")
            .items(&options)
            .interact()?;

        Ok(selections)
    }
}

mod prompter {
    use super::Result;
    use anyhow::{anyhow, Context};
    use std::io::{BufRead, Write};

    /// Function `prompt_for_token` prompts the user to enter a GitHub API token and returns it.
    ///
    /// # Panics
    ///
    /// This function panics if it is unable to prompt for the token in a secure manner.
    pub(crate) fn prompt_for_token() -> Result<String> {
        let token = rpassword::prompt_password("Enter token: ")
            .with_context(|| "Failed to prompt for token securely")?;

        Ok(token)
    }

    /// Function `prompt_user_input` prompts the user to enter a value and returns it.
    ///
    /// # Arguments
    ///
    /// * `message` - A message to display to the user when prompting for input.
    ///
    /// # Panics
    ///
    /// This function panics if it is unable to prompt for input in a secure manner.
    pub(crate) fn prompt_user_input(message: &str) -> Result<String> {
        print!("{}", message);
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    #[allow(dead_code)]
    pub(crate) fn prompt_for_privacy() -> Result<bool> {
        // Prompt the user to enter the privacy setting for the repository.
        println!("Should the repository be private? [y/n]");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|_| "Failed to read input".to_owned())
            .unwrap();

        // Parse the input as a boolean value.
        match input.trim().to_lowercase().as_ref() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(anyhow!("Invalid input, please enter y or n".to_owned())),
        }
    }
}

pub(crate) mod github {

    use super::{Result, ERROR_ICON, SUCCESS_ICON};
    use anyhow::anyhow;
    use indicatif::{ProgressBar, ProgressStyle};
    use reqwest::header::{self, HeaderValue};
    use serde::Deserialize;
    use serde_json::{json, Value};

    #[derive(Debug, Deserialize, Clone)]
    pub(crate) struct Repo {
        pub name: String,
        pub url: String,
        // private or isPrivate
        #[serde(rename = "private", skip_serializing_if = "Option::is_none")]
        pub private: Option<bool>,
    }
    ///
    /// [See docs] https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#list-repositories-for-the-authenticated-user
    ///
    /// ```
    /// $ curl -L \
    ///  -H "Accept: application/vnd.github+json" \
    ///  -H "Authorization: Bearer <YOUR-TOKEN>"\
    ///  -H "X-GitHub-Api-Version: 2022-11-28" \
    /// https://api.github.com/user/repos
    /// ```
    ///
    /// The `visibility` parameter can have one of the following values: `all | public | private | internal`
    pub(crate) async fn get_repos_request(_username: &str, pat_token: &str) -> Result<Vec<Repo>> {
        let visibility = String::from("all");
        let include_forks = false;

        let mut repositories = Vec::new();

        let mut page_number = 1;

        // Create a progress bar with a spinner style.
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("/|\\- ")
                .template("{spinner:.green} {msg}")?,
        );

        let mut params = vec![
            ("Visibility", visibility.as_str()),
            ("affiliation", "owner,collaborator"),
            ("per_page", "100"),
        ];
        match include_forks {
            true => params.push(("type", "all")),
            false => params.push(("fork", "false")),
        }

        // Loop until all pages have been fetched.
        'l: loop {
            // Show a message indicating that we are fetching the next page of repositories.
            progress_bar.set_message(format!("Fetching page {}", page_number));
            if page_number >= 3 {
                break 'l; // 300 items. 100 is max limit per page.
            }

            // Get the next page of repositories from GitHub.
            let client = reqwest::Client::new();
            let response = match client
                .get(
                    &(format!(
                        "https://api.github.com/user/repos?page={page}&per_page=100",
                        page = page_number,
                    )),
                )
                .header(header::ACCEPT, "application/vnd.github+json")
                .header(header::USER_AGENT, env!("CARGO_PKG_NAME"))
                .header(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", pat_token))?,
                )
                .send()
                .await
            {
                Ok(it) => it,
                Err(err) => {
                    let msg = format!("Failed to fetch page {}: {}\n", page_number, err);
                    progress_bar.finish_with_message(msg);
                    break 'l; // return Err(anyhow!(msg));
                }
            };
            if !response.status().is_success() {
                return Err(anyhow!(
                    "{ERROR_ICON} Failed to fetch repositories: {err:?}",
                    err = response.text().await?
                ));
            }

            let text = response.text().await?;
            let page_repositories: Vec<Repo> = serde_json::from_str(&text)?;
            // If there are no more pages, break the loop.
            if page_repositories.is_empty() {
                let msg = format!("{SUCCESS_ICON} All repositories fetched!",);
                progress_bar.finish_with_message(msg);
                break 'l;
            }

            // Add the repositories from the current page to the vector.
            repositories.extend(page_repositories);

            // Update the progress bar to indicate that we are fetching the next page.
            progress_bar.inc(1);
            page_number += 1;
        } // Once all repositories have been fetched, the progress bar is finished.

        println!(
            "{SUCCESS_ICON} Fetched details of `{count}` repos successfully!",
            count = repositories.len()
        );

        Ok(repositories)
    }

    /// Command to make the repository private:
    ///
    /// ```
    /// curl -H "Authorization: token <your PAT>" -X PATCH https://api.github.com/repos/<your username>/<your repository name> -d '{"private": true}'
    /// ```
    /// # Reference
    ///
    /// ```shell
    /// curl -L \ -X PATCH \ -H "Accept: application/vnd.github+json" \ -H "Authorization: Bearer <YOUR-TOKEN>" \ -H "X-GitHub-Api-Version: 2022-11-28" \ https://api.github.com/repos/OWNER/REPO \ -d '{"name":"Hello-World","description":"This is your first repository","homepage":"https://github.com","private":true,"has_issues":true,"has_projects":true,"has_wiki":true}'
    /// ```
    pub(crate) async fn post_request(
        repository: String,
        privacy: String,
        api_url: String,
        pat_token: String,
    ) -> Result<()> {
        let token = HeaderValue::from_str(&format!("token {}", pat_token))?;

        // Construct the request body.
        let body: Value = json!({
            "name": repository,
            "private": privacy, // 'true' || 'false'
            "auto_init": true,
        });

        // Send the API request.
        let client = reqwest::Client::new();
        let response = client
            .post(&api_url) // .patch(&api_url)
            .header(header::ACCEPT, "application/vnd.github.v3+json")
            .header(header::USER_AGENT, env!("CARGO_PKG_NAME"))
            .header(header::AUTHORIZATION, token)
            .body(body.to_string()) // Serialize the body to a JSON string.
            .send()
            .await?;

        // Check if the request was successful.
        if !response.status().is_success() {
            return Err(anyhow!(
                "{ERROR_ICON} Failed to update repository privacy setting: {err:?}",
                err = response.text().await?
            ));
        }

        println!("{SUCCESS_ICON} Repository privacy setting updated successfully!");

        Ok(())
    }
}

#[allow(dead_code)]
pub(crate) mod shell {
    use super::{Result, ERROR_ICON, SUCCESS_ICON};
    use crate::ApiResponse;
    use anyhow::{anyhow, Context};

    /// Command to make the repository private:
    ///
    /// ```
    /// curl -H "Authorization: token <your PAT>" -X PATCH https://api.github.com/repos/<your username>/<your repository name> -d '{"private": true}'
    /// ```
    fn post_request_curl(
        _repository: String,
        privacy: String,
        api_url: String,
        pat_token: Option<String>,
    ) -> Result<()> {
        let options = format!(r#"{{"private": {is_private}}}"#, is_private = privacy);
        let auth_header = format!("Authorization: token {token}", token = pat_token.unwrap(),);

        let cmd = std::process::Command::new("curl")
            .args(["-H", &auth_header, "-X", "PATCH", &api_url, "-d", &options])
            .output() // .spawn()
            .with_context(|| "curl command failed to start")?;

        // The API call was successful, and the response can be accessed here.
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        match serde_json::from_str::<ApiResponse>(&stdout) {
            Ok(response) if response.message == "Not Found" => {
                return Err(anyhow!(
                    "{ERROR_ICON} Failed to execute `curl` command: `{response:?}`",
                    response = response,
                ));
            }
            _ => (),
        }

        if !cmd.status.success() {
            return Err(anyhow!(
                "{ERROR_ICON} Failed to execute `curl` command: `{stderr:?}`",
                stderr = cmd.stderr,
            ));
        }
        println!("{SUCCESS_ICON} curl: {cmd}", cmd = cmd.status);

        Ok(())
    }
}
// // Validate the privacy and api.
// match privacy.trim().to_lowercase().as_ref() {
//     "true" | "false" => (),
//     _ => return Err(anyhow!("Invalid `privacy`:, enter `true` or `false`")),
// };
// let api_url_template =
//     "https://api.github.com/repos/<your username>/<your repository name>";
// if !api_url.contains( api_url_template .split("<") .collect::<Vec<_>>() .first() .unwrap()) {
//     return Err(anyhow!( "Internal error: `api_url` must be similar to `{help}`", help = api_url_template));
// }
