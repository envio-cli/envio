# Usage Guide

## Getting Started

### What are Profiles?

A profile is a collection of environment variables for a specific use case. For example, you might have a profile called "production" with your production API keys, and another called "development" with your local development settings.

You can choose from the following encryption methods for your profiles:

- **No Encryption (`none`)**: Stores the profile in plain text. Not recommended for sensitive data, but useful for non-sensitive configuration or testing.
- **Passphrase Encryption (`passphrase`)**: Encrypts the profile using a password you provide. You'll need to enter this password each time you access the profile.
- **Age Encryption (`age`)** _(BETA)_: Uses the [age](https://crates.io/crates/age) encryption library. Similar to passphrase encryption but uses the age encryption format.
- **GPG Encryption (`gpg`)**: Uses your GPG keys to encrypt the profile. A good option if you already use GPG, and you don't need to remember a separate password. (Unix only)

**Note**: Once you choose an encryption method for a profile, it cannot be changed

---

## Commands

### Creating Profiles

#### Basic Creation

This will create a new profile with no environment variables:

```bash
envio create <PROFILE_NAME>
```

Or use the `new` alias:

```bash
envio new <PROFILE_NAME>
```

This will prompt you to:

- Choose an encryption method
- Enter your encryption key if using passphrase/age, or select a GPG key if using GPG

#### Create with Environment Variables

Add variables directly when creating the profile using the `-e` or `--envs` flag:

```bash
envio create <PROFILE_NAME> -e API_KEY=secret123 DATABASE_URL=postgres://localhost
```

If you only provide the key without a value, the tool will prompt you for it:

```bash
envio create <PROFILE_NAME> -e API_KEY DATABASE_URL
```

You can use a mix of both:

```bash
envio create <PROFILE_NAME> -e API_KEY=secret DATABASE_URL
```

In this case, you'll be prompted to enter the value for `DATABASE_URL` and the value for `API_KEY` will be picked up automatically

#### Create from a File

Import variables from an existing file using the `-f` or `--from-file` flag:

```bash
envio create <PROFILE_NAME> -f <PATH_TO_FILE>
```

You'll be able to select which variables to include in your new profile.

#### Add Comments and Expiration Dates

Add comments to help you remember what each variable is for using the `-c` or `--comments` flag:

```bash
envio create <PROFILE_NAME> -e API_KEY=secret123 -c
```

Add expiration dates to track when credentials expire using the `-x` or `--expires` flag:

```bash
envio create <PROFILE_NAME> -e API_KEY=secret123 -x
```

You can combine both:

```bash
envio create <PROFILE_NAME> -e API_KEY=secret123 -c -x
```

For each variable, you'll be prompted to enter a comment and/or expiration date.

#### Specify Encryption Method

Instead of being prompted, you can choose your encryption method upfront using the `-k` or `--cipher-kind` flag:

```bash
envio create <PROFILE_NAME> -k passphrase
envio create <PROFILE_NAME> -k age
envio create <PROFILE_NAME> -k gpg
envio create <PROFILE_NAME> -k none
```

#### Add a Description

Include a description to remember what the profile is for:

```bash
envio create <PROFILE_NAME> -d "profile description goes here"
```

#### Complete Example

Here's a full example with all options:

```bash
envio create dev \
  -d "Development API keys and database credentials" \
  -k passphrase \
  -e API_KEY DATABASE_URL \
  -c -x
```

### Listing Profiles

```bash
envio list
envio ls # short alias
```

This will output a table with profile names, descriptions (if set), encryption methods, and creation/update timestamps in a formatted table.

A `--no-pretty-print` option is available to plain text output:

```bash
envio list --no-pretty-print
```

This outputs a simple list format that's easier to parse in scripts.

### Viewing Profile Contents

Display all environment variables in a profile:

```bash
envio show <PROFILE_NAME>
```

This shows a formatted table with variable names and values.

Show with comments using the `-c` or `--show-comments` flag:

```bash
envio show <PROFILE_NAME> -c
```

This adds a "Comment" column to the output showing any comments you've added to variables.

Show with expiration dates using the `-x` or `--show-expiration` flag:

```bash
envio show <PROFILE_NAME> -x
```

This adds an "Expiration Date" column showing when each variable expires (if set).

Show both comments and expiration dates:

```bash
envio show <PROFILE_NAME> -c -x
```

plain output using the `--no-pretty-print` flag:

```bash
envio show <PROFILE_NAME> --no-pretty-print
```

This outputs in `KEY=VALUE` format, one per line, which is perfect for sourcing in shell scripts or parsing programmatically.

### Modifying Profiles

#### Adding or Updating Variables

Use `set` to add new variables or update existing ones:

```bash
envio set <PROFILE_NAME> API_KEY=newvalue123
```

If `API_KEY` already exists in the profile, it will be updated with the new value. If it doesn't exist, it will be added.

Set multiple variables at once:

```bash
envio set <PROFILE_NAME> API_KEY=value1 DATABASE_URL=value2
```

You can set as many variables as you want in a single command, separated by spaces.

If you only provide the key, envio will prompt for the value:

```bash
envio set <PROFILE_NAME> NEW_API_KEY
```

Add/update comments and expiration dates using the `-c` or `--comments` and `-x` or `--expires` flags:

```bash
envio set <PROFILE_NAME> API_KEY=value123 -c -x
```

For each variable, you'll be prompted to enter a comment and/or expiration date.

#### Removing Variables

Remove one or more variables:

```bash
envio unset <PROFILE_NAME> API_KEY
envio unset <PROFILE_NAME> API_KEY DATABASE_URL
```

### Using Profiles

#### Loading into Terminal

Load a profile into your current terminal session:

```bash
envio load <PROFILE_NAME>
```

- On Unix systems, you'll need to reload your shell to apply changes
- On Windows, you'll need to restart your shell to apply changes

#### Running Commands with a Profile

Run a command using a profile's environment variables without loading them permanently:

```bash
envio run <PROFILE_NAME> -- npm run dev
envio run <PROFILE_NAME> -- python app.py
```

The `--` separates the profile name from the command. Everything after `--` is executed with the profile's environment variables.

#### Unloading Profiles

On Unix systems, unload the currently loaded profile:

```bash
envio unload
```

On Windows, you need to specify which profile to unload:

```bash
envio unload <PROFILE_NAME>
```

Restart your shell to apply changes.

### Importing and Exporting

#### Importing Profiles

Import from a local file:

```bash
envio import <PATH_TO_FILE>
```

Import from a URL:

```bash
envio import <URL>
```

Specify a custom name for the imported profile using the `-n` or `--profile-name` flag:

```bash
envio import <FILE_OR_URL> -n <PROFILE_NAME>
```

If you don't specify a name, `envio` will use the filename (without extension) or default to "imported"

#### Exporting Profiles

Export all variables to a file:

```bash
envio export <PROFILE_NAME>
```

This creates a `.env` file in your current directory by default.

Specify a custom output file using the `-o` or `--output-file-path` flag:

```bash
envio export <PROFILE_NAME> -o <PATH_TO_FILE>
```

Export only specific variables (comma-separated list of keys) using the `-k` or `--keys` flag:

```bash
envio export <PROFILE_NAME> -k API_KEY,DATABASE_URL
```

Interactively select which variables to export:

```bash
envio export <PROFILE_NAME> -k select
```

### Deleting Profiles

Remove a profile permanently:

```bash
envio delete <PROFILE_NAME>
```

Or use the `remove` alias:

```bash
envio remove <PROFILE_NAME>
```

### Interactive TUI

Launch the interactive terminal user interface:

```bash
envio tui
```

This opens a visual interface where you can manage profiles and create/edit variables with a more user-friendly experience.

> [!WARNING]
> The TUI is in beta so expect some bugs

### Shell Completions

Get shell completion scripts for easier command-line usage:

```bash
envio completion <SHELL>
```

Available shells: `bash`, `zsh`, `fish`, `powershell`

---

### Version Information

Check the installed version:

```bash
envio version
```

Get detailed version information using the `-v` or `--verbose` flag:

```bash
envio version -v
```

---

### Diagnostic Information

If you encounter issues, use the `--diagnostic` flag to generate diagnostic information for bug reports:

```bash
envio --diagnostic <COMMAND>
```

For example:

```bash
envio --diagnostic create <PROFILE_NAME>
```

This will output system and environment information useful for debugging issues.

## Getting Help

For any command, add `--help` to see usage information:

```bash
envio create --help
envio set --help
envio --help
```

## Environment Variables

#### `ENVIO_KEY`

Set this environment variable to provide your encryption key without being prompted. This is useful for automation, scripts, and CI/CD pipelines

- passphrase/age encryption: your encryption key
- GPG encryption: your GPG key fingerprint

For example:

```bash
ENVIO_KEY="supersecretkey" envio create <PROFILE_NAME>
ENVIO_KEY="0123456789ABCDEF..." envio show <PROFILE_NAME>
ENVIO_KEY="helloworld" envio run <PROFILE_NAME> -- npm run dev
```
