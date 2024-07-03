# War Machine

War Machine is a tool for managing and installing services, tools, and libraries.

## Installation

1. On Github, create a personal access token with the `repo` scope. This is necessary to download our services from the private repos. You can create it at [https://github.com/settings/tokens](https://github.com/settings/tokens).
2. Set the `GITHUB_TOKEN` environment variable to your GitHub personal access token. It is a good idea to add this to your shell configuration file (e.g. `.zshrc`, `.bashrc`, etc.), but **keep it secret!**
   ```sh
   export GITHUB_TOKEN=your_token_here
   ```
3. Run the following command:
   ```sh
   curl -sS https://gixuqotpkdlrfbermgnf.supabase.co/storage/v1/object/public/dev/install.sh | sudo GITHUB_TOKEN=$GITHUB_TOKEN bash
   ```
