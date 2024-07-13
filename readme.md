# War Machine

War Machine is a tool for managing and installing services, tools, and libraries. Sort of like docker-compose, but also supports services that don't use docker.

Supported platforms (Contributions welcome!):
- MacOS
- Linux

## Installation

```sh
bash <(curl -sS "https://raw.githubusercontent.com/generative-ai-inc/war-machine/main/install.sh")
```

### Running
Set up your `war_machine.toml` configuration file. See [war_machine.toml](https://github.com/generative-ai-inc/war-machine/blob/main/war_machine.toml) for an example.

Then run:
```sh
wm run <command-name>
```

### Private Image Access

In your `war_machine.toml` file you need to add one or more registry credentials:

```toml
registry_credentials = [
  { registry = "ghcr.io", username = "GITHUB_USERNAME", password = "GITHUB_TOKEN" },
  { registry = "docker.io", username = "DOCKER_USERNAME", password = "DOCKER_PASSWORD" },
]
```

These credentials will be read from env variables, env file, or keyring. You can add them to the keyring with the `wm secret add` command. For example:

```sh
wm secret add GITHUB_USERNAME <github-username>
wm secret add GITHUB_TOKEN
```

Github Personal Access Tokens should have the following scopes:

- `read:packages`

### Bitwarden Secret Manager

To use the bitwarden secret manager, you need to have the BWS_ACCESS_TOKEN variable set. We recommend using the keyring to store this token. You can do this with the following command:

```sh
wm secret add BWS_ACCESS_TOKEN
```

## Suggestions

### Shell Autocomplete

#### Zsh

To add completions for zsh, execute the following:

```
mkdir -p ${ZDOTDIR:-~}/.zsh_functions
echo 'fpath+=${ZDOTDIR:-~}/.zsh_functions' >> ${ZDOTDIR:-~}/.zshrc
wm completions zsh > ${ZDOTDIR:-~}/.zsh_functions/_wm
```

#### Other Shells

In general, you can generate completions for any shell with the following command:

```sh
wm completions <shell>
```

If you are not sure what to do with the output of this command, the people from Alacritty have a good [guide](https://github.com/alacritty/alacritty/blob/master/INSTALL.md#shell-completions) on how to add shell completions to your shell. In the guide it is assumed that you are adding the completions for the `alacritty` command, but the process is similar for other commands, like `wm`.
