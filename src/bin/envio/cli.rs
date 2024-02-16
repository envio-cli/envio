use clap::Parser;

#[derive(Parser)]
/*
 * Clap Application
*/
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
/*
 * The Command enum is a list of all possible subcommands
 * When a subcommand is passed to the application, clap returns the corresponding enum variant
 * The enum variant then calls the run method which is implemented in the command.rs file.
 *
 * When adding a new subcommand, add it to this enum and then write a match arm in the run method
 * which is located in the command.rs file
*/
pub enum Command {
    #[command(
        name = "create",
        about = "Create a new profile",
        override_usage = "envio create <PROFILE_NAME> [OPTIONS]"
    )]
    Create {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = false, long = "file-to-import-envs-from", short = 'f')]
        envs_file: Option<String>,
        #[arg(
            required = false,
            long = "envs",
            short = 'e',
            value_delimiter = ' ',
            num_args = 1..,
        )]
        envs: Option<Vec<String>>,
        #[arg(required = false, long = "gpg-key-fingerprint", short = 'g')]
        gpg: Option<String>,
    },
    #[command(
        name = "add",
        about = "Add envionment variables to a profile",
        override_usage = "envio add <PROFILE_NAME> [OPTIONS]"
    )]
    Add {
        #[arg(required = true)]
        profile_name: String,
        #[arg(
            required = true,
            long = "envs",
            short = 'e',
            value_delimiter = ' ',
            num_args = 1..,
        )]
        envs: Vec<String>,
    },
    #[command(
        name = "load",
        about = "Load all environment variables in a profile for use in your terminal sessions"
    )]
    Load {
        #[arg(required = true)]
        profile_name: String,
    },
    #[cfg(target_family = "unix")]
    #[command(name = "unload", about = "Unload a profile")]
    Unload,
    #[cfg(target_family = "windows")]
    #[command(name = "unload", about = "Unload a profile")]
    Unload {
        #[arg(required = true)]
        profile_name: String,
    },
    #[command(
        name = "launch",
        about = "Run a command with the environment variables from a profile",
        override_usage = "envio launch <PROFILE_NAME> [OPTIONS]"
    )]
    Launch {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = true, long = "command", short = 'c')]
        command: String,
    },
    #[command(
        name = "remove",
        about = "Remove a environment variable from a profile",
        override_usage = "envio remove <PROFILE_NAME> [OPTIONS]"
    )]
    Remove {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = false, long = "envs-to-remove", short = 'e', value_delimiter = ' ', num_args = 1..)]
        envs: Option<Vec<String>>,
    },
    #[command(
        name = "list",
        about = "List all the environment variables in a profile or all the profiles currenty stored"
    )]
    List {
        #[arg(required = false, long = "profiles", short = 'p')]
        profiles: bool,
        #[arg(required = false, long = "profile-name", short = 'n')]
        profile_name: Option<String>,
        #[arg(required = false, long = "no-pretty-print", short = 'v')]
        no_pretty_print: bool,
    },
    #[command(
        name = "update",
        about = "Update environment variables in a profile",
        override_usage = "envio update <PROFILE_NAME> [OPTIONS]"
    )]
    Update {
        #[arg(required = true)]
        profile_name: String,
        #[arg(
            required = true,
            long = "envs",
            short = 'e',
            value_delimiter = ' ',
            num_args = 1..,
        )]
        envs: Vec<String>,
    },
    #[command(
        name = "export",
        about = "Export a profile to a file if no file is specified it will be exported to a file named .env",
        override_usage = "envio export <PROFILE_NAME> [OPTIONS]"
    )]
    Export {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = false, long = "file-to-export-to", short = 'f')]
        file: Option<String>,
    },
    #[command(
        name = "import",
        about = "Download a profile over the internet and import it into the system or import a locally stored profile into your current envio installation",
        override_usage = "envio import <PROFILE_NAME> [OPTIONS]"
    )]
    Import {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = false, long = "file-to-import-from", short = 'f')]
        file: Option<String>,
        #[arg(required = false, long = "url", short = 'u')]
        url: Option<String>,
    },
    #[command(name = "version", about = "Print the version")]
    Version {
        #[arg(required = false)]
        verbose: Option<bool>,
    },
}
