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

use anyhow::{anyhow, Context};

use serde::Deserialize;
use std::io::{BufRead, Write};
use std::process::Command;

type Result<T> = anyhow::Result<T, anyhow::Error>;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    message: String,
    documentation_url: String,
}

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
// let command = r#"curl -L \
// -X PATCH \
// -H "Accept: application/vnd.github+json" \
// -H "Authorization: Bearer <YOUR-TOKEN>" \
// -H "X-GitHub-Api-Version: 2022-11-28" \
// https://api.github.com/repos/OWNER/REPO \
// -d '{"name":"Hello-World","description":"This is your first repository","homepage":"https://github.com","private":true,"has_issues":true,"has_projects":true,"has_wiki":true}'"#;
fn main() -> Result<()> {
    // Load environment vairables from .env file.
    dotenv::dotenv().ok();
    let green_check_unicode = "\u{2705}"; // ✅
    let red_x_unicode = "\u{274C}"; // ❌

    let username = prompt_user_input("Enter username: ")?;
    let repository = prompt_user_input("Enter repository: ")?;

    // Get personal access token.
    let pat_token = std::env::var("PAT_TOKEN")
        .map(|token| match token.is_empty() {
            true => prompt_for_token().unwrap(),
            false => token,
        })
        .unwrap_or_else(|_| prompt_for_token().unwrap());

    let auth = format!("Authorization: token {token}", token = pat_token,);
    let api_url = format!(
        r#"https://api.github.com/repos/{username}/{repository}"#,
        username = username,
        repository = repository,
    );
    let is_private = 'l: loop {
        let input = prompt_user_input("Make it private?: (true/false) ")
            .unwrap_or_else(|_| "false".to_owned());
        match input == "true" || input == "false" {
            true => break 'l input,
            false => println!("{red_x_unicode} Please enter either `true` or `false`"),
        }
    };
    let options = format!("{{\"private\": {is_private}}}", is_private = is_private);

    let cmd = Command::new("curl")
        .args(["-H", &auth, "-X", "PATCH", &api_url, "-d", &options])
        .output() // .spawn()
        .with_context(|| "curl command failed to start")?;

    // The API call was successful, and the response can be accessed here.
    if let Ok(response) = serde_json::from_str::<ApiResponse>(&String::from_utf8_lossy(&cmd.stdout))
    {
        if response.message == "Not Found" {
            return Err(anyhow!(
                "{red_x_unicode} Failed to execute `curl` command: `{err:?}`",
                err = response,
            ));
        }
    }

    if !cmd.status.success() {
        return Err(anyhow!(
            "{red_x_unicode} Failed to execute `curl` command: `{stderr:?}`",
            stderr = cmd.stderr,
        ));
    }

    println!("{green_check_unicode} curl: {cmd}", cmd = cmd.status);

    Ok(())
}

/// Function `prompt_for_token` prompts the user to enter a GitHub API token and returns it.
///
/// # Panics
///
/// This function panics if it is unable to prompt for the token in a secure manner.
fn prompt_for_token() -> Result<String> {
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
fn prompt_user_input(message: &str) -> Result<String> {
    print!("{}", message);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Open an interactive shell like repl.
// let cmd = Command::new("sh")
//     .args(["echo", "hello"])
//     .output()
//     // .spawn()
//     .expect("sh command failed to start");
// let output = cmd.wait_with_output().unwrap();
// println!("{}", String::from_utf8_lossy(&output.stdout.to_vec()));

// Finally, in unit tests, you might want to pass a Cursor, which implements BufRead. In that case, you can use read_password_from_bufread and prompt_password_from_bufread:
//
// use std::io::Cursor;
//
// let mut mock_input = Cursor::new("my-password\n".as_bytes().to_owned());
// let password = rpassword::read_password_from_bufread(&mut mock_input).unwrap();
// println!("Your password is {}", password);
//
// let mut mock_input = Cursor::new("my-password\n".as_bytes().to_owned());
// let mut mock_output = Cursor::new(Vec::new());
// let password = rpassword::prompt_password_from_bufread(&mut mock_input, &mut mock_output, "Your password: ").unwrap();
// println!("Your password is {}", password);
