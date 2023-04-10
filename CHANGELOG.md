# Change Log
# v0.5.0
## Features
* Add GPG encryption for user profiles, See [Usage](./docs/usage.md)
* Add flags for using the CLI, See [Usage](./docs/usage.md)

## Bug Fixes
* Fix issue #17 where envio assumed shell config file was in home directory and would panic if it was not found; envio now prompts users to pass in their shell config if it cannot find it

* Fix bug where envio would exit without doing its first time setup routine if it could not find the shell config; envio now checks for the config directory to determine if it is a fresh install or not

## Other
* Switched to `age` from `magic_crypt` for passphrase encryption method #13

This document records all notable changes to [envio](https://github.com/humblepenguinn/envio).
# v0.4.1
## Bug Fixes
* Fix slow startup times due to slow update fetching

# v0.4.0
## Features
* Add new argument `--no-pretty-print` to `envio list` command, See [Usage](./docs/usage.md)
* Add support for `fish` shell #9 
* 
## Bug Fixes
* Fix Security Vulnerability, See [Here](./docs/envio-profile-loading-update.md)

## Other
* Fix readme typo by @erjanmx in https://github.com/humblepenguinn/envio/pull/10
* docs: add erjanmx as a contributor for doc by @allcontributors in https://github.com/humblepenguinn/envio/pull/11
* Bump tokio from 1.26.0 to 1.27.0 by @dependabot in https://github.com/humblepenguinn/envio/pull/12

# v0.3.0
# Features
* Users can now create new profiles using files and also pass in environment variables #8 
* Added support for the `fish` shell #9 

# Bug Fixes
* Both the config and profiles directory are created at startup, if they do not exist #8 

# Other
* docs: add Vojtch159 as a contributor for doc by @allcontributors in https://github.com/humblepenguinn/envio/pull/7
* Bump clap_complete from 4.1.5 to 4.2.0 by @dependabot in https://github.com/humblepenguinn/envio/pull/4
* Bump reqwest from 0.11.14 to 0.11.16 by @dependabot in https://github.com/humblepenguinn/envio/pull/5

# v0.2.0
## Features
* In addition to being able to load and unload profiles in current terminal sessions, users can now use the `envio launch` sub command to launch programs with specific profiles see [Usage](./docs/usage.md)
* envio now automatically looks for updates

## Bug Fixes
* Users do not need to type in their key twice when modifying environment variables in a profile

## Other
* Update [usage.md](./docs/usage.md) by @Vojtch159 in https://github.com/humblepenguinn/envio/pull/1
* Update [usage.md](./docs/usage.md) to include usage of `launch` sub command
* Update [CODEOWNERS](./CODEOWNERS)
* Update [README](./README.md) with new `gif` showing the new `launch` sub command

## New Contributors
* @Vojtch159 made their first contribution in https://github.com/humblepenguinn/envio/pull/1

# v0.1.0

- Inital Release


