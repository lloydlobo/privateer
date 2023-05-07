//! To get your repositories on Git in Rust CLI, you can use the Git command-line interface (CLI) tool called `git`. Here are the steps to follow:
//!
//! 1. Open your terminal or command prompt and navigate to the directory where you want to clone the repository.
//!
//! 2. Clone the repository by running the command `git clone <repository URL>` where the repository URL is the URL of the repository you want to clone.
//!
//! 3. Once the repository is cloned, navigate to the cloned repository directory by running the command `cd <repository name>`.
//!
//! 4. To edit the privacy settings of your repository, you can use the Git API or the web interface provided by your Git hosting provider (such as GitHub, GitLab, or Bitbucket). Here's an example of how to make a repository private using the GitHub API:
//!
//!    - First, you'll need to generate a personal access token (PAT) on GitHub by going to your account settings and selecting "Developer settings" > "Personal access tokens" > "Generate new token". Make sure to give the token the necessary permissions to modify repositories.
//!
//!    - Next, run the following command to make the repository private:
//!
//!      ```
//!      curl -H "Authorization: token <your PAT>" -X PATCH https://api.github.com/repos/<your username>/<your repository name> -d '{"private": true}'
//!      ```
//!
//!      Replace `<your PAT>` with your personal access token, `<your username>` with your GitHub username, and `<your repository name>` with the name of your repository.
//!
//!    - If you want to make multiple repositories private or public, you can create a script that loops through a list of repositories and makes the necessary changes using the Git API or the web interface provided by your Git hosting provider.
//!
//!    - Alternatively, you can use a third-party tool like the GitHub CLI (`gh`) to manage your repositories from the command line. `gh` provides an easy-to-use interface for managing repositories, including creating, cloning, and modifying them.

use anyhow::anyhow;

use serde::Deserialize;

pub(crate) type Result<T> = anyhow::Result<T, anyhow::Error>;
// Unicode symbols for success and error messages.
pub(crate) static SUCCESS_ICON: &str = "\u{2705}"; // ✅ green_check_unicode.
pub(crate) static ERROR_ICON: &str = "\u{274C}"; // ❌ red_x_unicode.

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    message: String,
    documentation_url: String,
}

/// This function makes a PATCH request to the GitHub API to update the privacy settings of a repository.
///
/// To make a public repository private using a personal access token (PAT) on GitHub, you need to have the `repo` scope in your PAT. The `repo` scope allows the PAT to access and modify the repository, including changing its privacy settings.
///
/// Here are the steps to generate a PAT with the required scope:
///
/// 1. Go to your GitHub account settings and select "Developer settings" > "Personal access tokens" > "Generate new token".
///
/// 2. Give the token a name and select the `repo` scope.
///
/// 3. Click on "Generate token" to create the PAT.
///
/// Once you have generated the PAT, you can use it to make a public repository private by sending a
/// PATCH request to the GitHub API with the following payload:
///
/// ```
/// {
///   "private": true
/// }
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
///
#[tokio::main]
async fn main() -> Result<()> {
    // Load environment vairables from .env file.
    dotenv::dotenv().ok();

    // Prompt the user to enter the username and repository name.
    let username = prompter::prompt_user_input("Enter username: ")?;
    if username.is_empty() {
        return Err(anyhow!("{ERROR_ICON} `username` is required",));
    }
    let repository = prompter::prompt_user_input("Enter repository: ")?;
    if repository.is_empty() {
        return Err(anyhow!("{ERROR_ICON} `password` is required",));
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

    // Construct the Authorization header and API URL.
    let api_url = format!(
        r#"https://api.github.com/repos/{username}/{repository}"#,
        username = username,
        repository = repository,
    );

    // Prompt the user to enter the privacy setting for the repository.
    let privacy = 'l: loop {
        let input = prompter::prompt_user_input("Make it private?: (true/false) ")
            .unwrap_or_else(|_| "false".to_owned());
        match input == "true" || input == "false" {
            true => break 'l input,
            false => println!("{ERROR_ICON} Please enter either `true` or `false`"),
        }
    };

    github::post_request(repository, privacy, api_url, Some(pat_token)).await?;

    Ok(())
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
    use crate::prompter::prompt_for_token;
    use reqwest::header::HeaderValue;
    use serde_json::{json, Value};
    // use reqwest::header::HeaderMap;

    /// Command to make the repository private:
    ///
    /// ```
    /// curl -H "Authorization: token <your PAT>" -X PATCH https://api.github.com/repos/<your username>/<your repository name> -d '{"private": true}'
    /// ```
    /// # Reference
    /// ```shell
    /// curl -L \
    /// -X PATCH \
    /// -H "Accept: application/vnd.github+json" \
    /// -H "Authorization: Bearer <YOUR-TOKEN>" \
    /// -H "X-GitHub-Api-Version: 2022-11-28" \
    /// https://api.github.com/repos/OWNER/REPO \
    /// -d '{"name":"Hello-World","description":"This is your first repository","homepage":"https://github.com","private":true,"has_issues":true,"has_projects":true,"has_wiki":true}'
    /// ```
    pub(crate) async fn post_request(
        repository: String,
        privacy: String,
        api_url: String,
        pat_token: Option<String>,
    ) -> Result<()> {
        let token = pat_token
            .as_deref()
            .map(|token| HeaderValue::from_str(&prefix_token(token)))
            .unwrap_or_else(|| {
                HeaderValue::from_str(&prefix_token(prompt_for_token().unwrap().as_str()))
            })?;

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
            .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
            .header(reqwest::header::USER_AGENT, env!("CARGO_PKG_NAME"))
            .header(reqwest::header::AUTHORIZATION, token)
            .body(body.to_string()) // Serialize the body to a JSON string.
            .send()
            .await?;

        // Check if the request was successful.
        if response.status().is_success() {
            println!("{SUCCESS_ICON} Repository privacy setting updated successfully!");
        } else {
            println!(
                "{ERROR_ICON} Failed to update repository privacy setting: {:?}",
                response.text().await?
            );
        }

        Ok(())
    }

    fn prefix_token(token: &str) -> String {
        format!("token {}", token)
    }
}

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
