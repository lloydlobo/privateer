use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let token = "<YOUR-TOKEN>";
    let url = "https://api.github.com/repos/OWNER/REPO";
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    let body = r#"{"name":"Hello-World","description":"This is your first repository","homepage":"https://github.com","private":true,"has_issues":true,"has_projects":true,"has_wiki":true}"#;
    let response = client.patch(url).headers(headers).body(body).send()?;
    println!("{:?}", response);
    Ok(())
}

// Note that you will need to add the `reqwest` crate to your `Cargo.toml` file in order to use this code. You can do so by adding the following line to your `[dependencies]` section:

// ```toml
// reqwest = { version = "0.11", features = ["json"] }
// ```
//
// Also, be sure to replace `<YOUR-TOKEN>` with your actual GitHub API token before running the code.
