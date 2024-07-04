# War Machine

War Machine is a tool for managing and installing services, tools, and libraries.

## Installation

1. On Github, create a personal access token with the `repo` scope. This is necessary to download our services from the private repos. You can create it at [https://github.com/settings/tokens](https://github.com/settings/tokens).
2. Use your github token to install War Machine:

   ```sh
   curl -sS https://gixuqotpkdlrfbermgnf.supabase.co/storage/v1/object/public/dev/install.sh | sudo GITHUB_TOKEN=<your-github-token> bash
   ```

   Make sure to replace `<your-github-token>` with your actual github token.

3. Add your bitwarden secret manager token to War Machine with the following command:

   ```sh
   wm token add BITWARDEN_ACCESS_TOKEN
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

## Notes

After the installations completes, War Machine will keep a copy of your github token saved in the keyring. You can remove any token with the following command:

```sh
wm token remove <token-name>
```
