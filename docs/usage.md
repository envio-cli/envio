# Usage
Before reading the usage make sure you understand what `profiles` (See [profiles](https://github.com/humblepenguinn/envio#profiles)) are in envio

Each command of envio has its own set of flags that can be used to modify their behavior. These flags have a short form and a long form, and users can choose to use either of them. For instance, the `--file-to-import-envs-from` flag can be written as `-f`. Both forms of the flag are equivalent and can be used interchangeably. This document provides examples for each flag with both variants, to make it easy for users to remember and use them.

## Creating a Profile
To use `envio`, you first need to create a profile.

To create a profile, simply use the `envio create` command. By default, envio uses `passphrase` encryption with the help of the [`age`](https://crates.io/crates/age) crate to protect your profile. To create a profile with the default `passphrase` encryption method, use the command:

```sh
$ envio create <profile_name>
```
Replace `<profile_name>` with the the name of the profile you want to use.

```sh
$ envio create myprofile
```

This command will create a new profile called `myprofile` and prompt you to type in a key. This key will be used later on to encrypt and decrypt your profile, so make sure to remember it.

If you want to use a different encryption method, envio also supports `GPG` encryption. To create a profile with GPG encryption, use the command:
```sh
$ envio create <profile_name> -g <gpg_key_fingerprint>
```

where `<gpg_key_fingerprint>` is the fingerprint of the `GPG` key you want to use. Alternatively, you can use the command:

```sh
$ envio create <profile_name> -g select
```

This command will prompt you with a list of all the public `GPG` keys available on your system, and you can select the one you want to use.

It's important to note that different operating systems require different dependencies to use `GPG` encryption with envio.

For `Windows` users, you will need to have [`GPG4Win`](https://www.gpg4win.org/) installed on your system in order to use GPG encryption with envio.

For `Linux` users, the required dependencies for using `GPG` encryption with envio may vary depending on your specific distribution and setup. To determine which dependencies you need, we recommend visiting the `gpgme` crate's GitHub page at https://github.com/gpg-rs/gpgme.

On this page, you can find more information about how to install the required dependencies to use the crate. Since the package names may vary on different Linux distributions, it's important to carefully read the documentation and follow the installation instructions for your particular setup.

For `macOS` users, you will need to have `gnupg` and `gpgme` installed on your system. You can install these dependencies using `Homebrew`, a popular package manager for `macOS`.

## Adding Environment Variables to a Profile

Once you have created a profile, you can use the `envio add <profile_name> --envs <key>=<value>` command to add enviorment variables to it. to add a variable named DATABASE_URL with a value of postgres://localhost/mydb, you would run the command:

```sh
$ envio add myprofile --envs DATABASE_URL=postgres://localhost/mydb
```

Or

```sh
$ envio add myprofile -e DATABASE_URL=postgres://localhost/mydb
```

You can add multiple environment variables too

```sh
$ envio add myprofile -e 'DATABASE_URL=postgres://localhost/mydb MY_VERY_SECRETIVE_KEY=1234'
```

Make sure you include the environment variables you want to add in between quotes seperated by a space

Instead of creating a profile first and then adding environment variables, users can directly add environment variables when creating a profile using the `create` command (See [create](#creating-a-profile)):

```sh
$ envio create <profile_name> -e <key>=<value>
# OR
$ envio create <profile_name> --envs <key>=<value>
```

Or if you want to use the `GPG` encryption
```sh
$ envio create <profile_name> -e <key>=<value> -g select
```

They can also import environment variables from a file too

```sh
$ envio create <profile_name> -f <file_name>
# OR
$ envio create <profile_name> --file-to-import-envs-from <file_name>
```

You can view all the environment variables in your profile using the `list` command (See [List](#list) command)

## Updating Environment Variables in a Profile

To edit an existing variable, you can use the `envio update <profile_name> --env <key>=<new_value>` command. For example, to change the value of the DATABASE_URL variable in the `myprofile` profile to postgres://myhost/mydb, you would run the command:

```sh
$ envio update myprofile --envs DATABASE_URL=postgres://myhost/mydb
# OR
$ envio update myprofile -e DATABASE_URL=postgres://myhost/mydb
```

## Removing Environment Variables in a Profile

To remove a variable from a profile, use the `envio remove <profile_name> --envs-to-remove <key>` command. For example, to remove the DATABASE_URL variable from the `myprofile` profile, you would run the command:

```sh
$ envio remove myprofile --envs-to-remove MY_VERY_SECRETIVE_KEY
# OR
$ envio remove myprofile -e MY_VERY_SECRETIVE_KEY
```

## Delete a Profile
If a user wants to delete a entire profile they can use the `envio remove` command without any flags:

```sh
$ envio remove <profile_name>
```

## Loading a Profile

You can use the `envio load <profile_name>` command to load the profile and make the environment variables available in your terminal session.

```sh
$ envio load <profile_name>
```

On `Windows`, users just need to reload their shell and they can start using their environment variables as before. However, on `Unix-based` operating systems, a new approach has been implemented to load the environment variables securely. Whenever users open their shell, envio now asks users for the key used for the profile that was loaded. They have to type in the key to access their environment variables, which are then stored in a temporary file and sourced in the current session. This ensures that the environment variables are loaded securely and are not accessible to anyone without the correct key.

Now,
```sh
$ echo $DATABASE_URL
```

## Unloading a Profile

On `Windows`, to unload a profile from the current session, run the command:
```sh
$ envio unload <profile_name>
```

On `Unix-based` operating systems, to unload a profile from the current session, run the `envio unload` command without any arguments:
```sh
$ envio unload
```

## Launching a Program with a Profile

The `envio launch` command allows you to run a program using a specific profile. This is useful when you need to switch between different sets of environment variables for different projects or environments.

To use this command, simply run:

```sh
$ envio launch <profile_name> <program>
```

where `<profile_name>` is the name of the profile you want to use and `<program>` is the name of the program you want to run and the arguments that you want to pass to the program.

For example, if you have a profile called `dev` with a set of environment variables specific to your development environment, you can run your program with these variables using the following command:

```sh
$ envio launch dev python my_program.py
```

This will run the python my_program.py command with the environment variables from the dev profile.

## Importing Profiles from the Internet
Users can download profiles over the internet using the following command:

```sh
$ envio import <profile_name_to_save_as> -u <url>
# OR
$ envio import <profile_name_to_save_as> --url <url>
```

`url` should point to a valid URL where the profile can be downloaded. The `profile_name_to_save_as` argument specifies the name of the profile to save as.

## Importing Profiles from a File

Users can also import profiles from a file using the same command as before, but with the url argument replaced by a file path:

```sh
$ envio import <profile_name_to_save_as> -f <file_path>
# OR
$ envio import <profile_name_to_save_as> --file-to-import-from <file_path>
```

`file_path` should point to a valid file path where the profile can be found. The `profile_name_to_save_as` argument specifies the name of the profile to save as.


## Exporting Environment Variables from a Profile
To export all environment variables from a profile, users can run the following command:

```sh
$ envio export <profile_name> -f <file_to_export_to>
# OR
$ envio export <profile_name> --file-to-export-to <file_to_export_to>
```

If the `file_to_export_to` argument is not specified, the command will export the environment variables to a file called `.env`

By exporting and importing environment variables, users can easily share configurations between different machines or team members, or save and load different configurations as needed.

## List
You can view all your existing profiles and also all the environment variables in a specific profile

To list all the existing profiles, users can run the following command:
```sh
$ envio list -p
# OR
$ envio list --profiles
```

To list all the environment variables in a profile, users can run the following command:
```sh
$ envio list -n <profile_name>
# OR
$ envio list --profile-name <profile_name>
```

Users can also view their profiles and environment variables in a profile without any visual formatting using the `--no-pretty-print` flag

To list all existing profiles:
```sh
$ envio list -p -v
# OR
$ envio list -p --no-pretty-print
```

To list all the environment variables in a profile:
```sh
$ envio list -n <profile_name> -v
```