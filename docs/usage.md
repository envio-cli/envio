# Usage
Before reading the usage make sure you understand what `profiles` (See [profiles](https://github.com/humblepenguinn/envio#profiles)) are in envio

## Creating a Profile

To use `envio`, you first need to create a profile. You can use the `envio create` command, followed by the name of the profile you want to create.

```sh
$ envio create myprofile
```

This will create a new profile named myprofile.

All profiles are encrypted so when you create a profile with `envio create`, you will be prompted to enter a key for the profile. This key will be used to encrypt and decrypt the environment variables in the profile. Whenever you run a command that manipulates the enviornment variables of a profile, you will need to provide the key for the profile in order to apply the changes.

If you want to create a profile and import all the enviornment variables from a file you can use the command below:

```sh
$ envio create <file> <profile_name_to_save_as>
```
This will create a profile with all the environment variables from your file. 

Lets say we have a file named '.env' and we want to create a profile named 'myprofile' which imports all the environment variables from the `.env` file, we can run the command below:

```sh
$ envio create .env myprofile
```

You can view all the environment variables in your profile using the `list` command (See [List](#list) command)

## Adding Environment Variables to a Profile

Once you have created a profile, you can use the `envio add <profile_name> <key>=<value>` command to add enviorment variables to it. to add a variable named DATABASE_URL with a value of postgres://localhost/mydb, you would run the command:

```sh
$ envio add myprofile DATABASE_URL=postgres://localhost/mydb
```

You can add multiple environment variables too

```sh
$ envio add myprofile DATABASE_URL=postgres://localhost/mydb MY_VERY_SECRETIVE_KEY=1234
```

Instead of creating a profile first and then adding environment variables, users can directly add environment variables when creating a profile using the `create` command (See [create](#creating-a-profile)):
```sh
$ envio create <profile_name> <key>=<value>
```
You can add multiple environment variables

So instead of first creating a profile named `myprofile` and then adding environment variables to it. We can directly run the command below:
```sh
$ envio create myprofile DATABASE_URL=postgres://localhost/mydb MY_VERY_SECRETIVE_KEY=1234
```

This will create a profile named `myprofile` with two environment variables, `DATABASE_URL` and `MY_VERY_SECRETIVE_KEY` with values `postgres://localhost/mydb` and `1234` respectively

## Updating Environment Variables in a Profile

To edit an existing variable, you can use the `envio update <profile_name> <key>=<new_value>` command. For example, to change the value of the DATABASE_URL variable in the `myprofile` profile to postgres://myhost/mydb, you would run the command:

```sh
$ envio update myprofile DATABASE_URL=postgres://myhost/mydb
```

## Removing Environment Variables in a Profile

To remove a variable from a profile, use the `envio remove <profile_name> <key>` command. For example, to remove the DATABASE_URL variable from the `myprofile` profile, you would run the command:

```sh
$ envio remove myprofile MY_VERY_SECRETIVE_KEY
```

## Loading a Profile

You can use the `envio load <profile_name>` command to load the profile and make the environment variables available in your terminal session.

```sh
$ envio load myprofile
```

On `Windows`, users just need to reload their shell and they can start using their environment variables as before. However, on `Unix-based` operating systems, a new approach has been implemented to load the environment variables securely. Whenever users open their shell, envio now asks users for the key used for the profile that was loaded. They have to type in the key to access their environment variables, which are then stored in a temporary file and sourced in the current session. This ensures that the environment variables are loaded securely and are not accessible to anyone without the correct key.

Now,
```sh
$ echo $DATABASE_URL
```

## Unloading a Profile

On `Windows`, to unload a profile from the current session, run the command:
```sh
$ envio unload myprofile
```

On `Unix-based` operating systems, to unload a profile from the current session, run the `envio unload` command without any arguments:
```sh
$ envio unload
```

## Launching a Program with a Profile

The `envio launch` command allows you to run a program using a specific profile. This is useful when you need to switch between different sets of environment variables for different projects or environments.

To use this command, simply run:

```sh
$ envio launch <profile> <program>
```

where `<profile>` is the name of the profile you want to use and `<program>` is the name of the program you want to run.

For example, if you have a profile called `dev` with a set of environment variables specific to your development environment, you can run your program with these variables using the following command:

```sh
$ envio launch dev python my_program.py
```
  
This will run the python my_program.py command with the environment variables from the dev profile.

## Importing Profiles from the Internet
Users can download profiles over the internet using the following command:

```sh
$ envio import <url> <profile_name_to_save_as>
```

The `url` argument should point to a valid URL where the profile can be downloaded. The `profile_name_to_save_as` argument specifies the name of the profile to save as.

## Importing Profiles from a File

Users can also import profiles from a file using the same command as before, but with the url argument replaced by a file path:

```sh
$ envio import <file_path> <profile_name_to_save_as>
```

The `file_path argument` should point to a valid file path where the profile can be found. The `profile_name_to_save_as` argument specifies the name of the profile to save as.

By exporting and importing environment variables, users can easily share configurations between different machines or team members, or save and load different configurations as needed.

## Exporting Environment Variables from a Profile
To export all environment variables from a profile, users can run the following command:

```sh
$ envio export <profile_name> <file_to_export_to>
```

If the `file_to_export_to` argument is not specified, the command will export the environment variables to a file called `.env`

## List
You can view all your existing profiles and also all the environment variables in a specific profile

To list all the existing profiles, users can run the following command:
```sh
$ envio list profiles
```

To list all the environment variables in a profile, users can run the following command:
```sh
$ envio list <profile_name>
```

Users can also view their profiles and environment variables in a profile without any visual formatting using the `-no-pretty-print` argument

To list all existing profiles:
```sh
$ envio list profiles -- --no-pretty-print
```

To list all the environment variables in a profile:
```sh
$ envio list <profile_name> -- --no-pretty-print
```
