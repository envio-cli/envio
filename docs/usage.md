# Usage

## Creating a Profile

To use `envio`, you first need to create a profile. You can use the `envio create` command, followed by the name of the profile you want to create.

```sh
$ envio create myprofile
```

This will create a new profile named myprofile.

All profiles are encrypted so when you create a profile with `envio create`, you will be prompted to enter a key for the profile. This key will be used to encrypt and decrypt the environment variables in the profile. Whenever you run a command that manipulates the enviornment variables of a profile, you will need to provide the key for the profile in order to apply the changes.

## Adding Environment Variables to a Profile

Once you have created a profile, you can use the `envio add <profile_name> <key>=<value>` command to add enviorment variables to it. to add a variable named DATABASE_URL with a value of postgres://localhost/mydb, you would run the command:

```sh
$ envio add myprofile DATABASE_URL=postgres://localhost/mydb
```

You can add multiple environment variables too

```sh
$ envio add myprofile DATABASE_URL=postgres://localhost/mydb MY_VERY_SECRETIVE_KEY=1234
```

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

Now,
```sh
$ echo $DATABASE_URL
```

## Unloading a Profile

To unload a profile from the current session, run the command:
```sh
$ envio unload myprofile
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

