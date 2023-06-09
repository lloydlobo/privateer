# privateer

**CLI Application for making a GitHub repository private**

This is a command-line interface (CLI) application that makes a GitHub repository private using a personal access token (PAT). The app is written in Rust programming language and uses the `serde` and `anyhow` crates.

## Prerequisites

- Rust programming language should be installed.
- A personal access token (PAT) with `repo` scope is required to modify the repository's privacy settings.

## Usage

1. Clone this repository:

   ```
   git clone https://github.com/username/repo-name.git
   ```

2. Change directory to the project's root:

   ```
   cd repo-name
   ```

3. Create a `.env` file and add your PAT token in the following format:

   ```
   PAT_TOKEN=<your-token-here>
   ```

4. Build the project:

   ```
   cargo build --release
   ```

5. Run the CLI app:

   ```
   ./target/release/make-private-repo
   ```

6. Enter the GitHub username and repository name when prompted.

```shell
Enter username: lloydlobo
Enter repository: gittidy
...
```

7. Choose whether to make the repository private or not by entering either `true` or `false`.

```shell
...
Make it private?: (true/false) true
✅ curl: exit status: 0
```

## Functionality

The app will send a PATCH request to the GitHub API with the personal access token (PAT) included in the `Authorization` header.
The request payload will contain the `private` field set to either `true` or `false`, depending on the user's input.

If the API call is successful, the repository will be made private.

## Further Information

For more information on how to create a personal access token (PAT), visit the [GitHub documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token).

For more information on how to make a repository private using the GitHub API, visit the [GitHub documentation](https://docs.github.com/en/rest/reference/repos#update-a-repository).

## License

This project is licensed under the terms of the [LICENSE].
