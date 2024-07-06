# War Machine

War Machine is a tool for managing and installing services, tools, and libraries.

## Installation

1. On Github, create a personal access token with the `repo` scope and `read:packages` scope. This is necessary to download our services from the private repos. You can create it at [https://github.com/settings/tokens](https://github.com/settings/tokens).
2. Run the installation script. It will ask you to enter your github token.

   ```sh
   curl -sS https://gixuqotpkdlrfbermgnf.supabase.co/storage/v1/object/public/dev/install.sh | sudo bash
   ```

3. Add your github username to War Machine with the following command:

   ```sh
   wm secret add GITHUB_USERNAME <github-username>
   ```

4. Add your bitwarden secret manager access token to War Machine with the following command:

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

## Notes

After the installations completes, War Machine will keep a copy of your github token as a secret saved in the keyring. You can remove any secret with the following command:

```sh
wm secret remove <secret-name>
```
