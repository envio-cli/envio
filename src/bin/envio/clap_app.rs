use clap::Parser;

#[derive(Parser, Debug)]
pub struct ClapApp {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[command(subcommand)]
    Profile(ProfileCommand),

    #[command(
        name = "set",
        about = "Set or update environment variables in a profile",
        override_usage = "envio set <PROFILE_NAME> <ENVS>... [OPTIONS]"
    )]
    Set {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = true, value_delimiter = ' ', num_args = 1..)]
        envs: Vec<String>,
        #[arg(long = "comments", short = 'c')]
        comments: bool,
        #[arg(long = "expiration-date", short = 'x')]
        expiration_date: bool,
    },

    #[command(
        name = "unset",
        about = "Remove an environment variable from a profile",
        override_usage = "envio unset <PROFILE_NAME> <KEYS>... [OPTIONS]"
    )]
    Unset {
        #[arg(required = true)]
        profile_name: String,
        #[arg(required = true, value_delimiter = ' ', num_args = 1..)]
        keys: Vec<String>,
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
        name = "run",
        about = "Run a command with the environment variables from a profile",
        override_usage = "envio run <PROFILE_NAME> -- <COMMAND>"
    )]
    Run {
        #[arg(required = true)]
        profile_name: String,
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },

    #[command(
        name = "import",
        about = "Import a profile from a file, URL, or .env file",
        override_usage = "envio import <SOURCE> [OPTIONS]"
    )]
    Import {
        #[arg(required = true)]
        source: String,
        #[arg(long = "profile-name", short = 'n')]
        profile_name: Option<String>,
    },

    #[command(
        name = "export",
        about = "Export a profile to a file",
        override_usage = "envio export <PROFILE_NAME> [OPTIONS]"
    )]
    Export {
        #[arg(required = true)]
        profile_name: String,
        #[arg(long = "to", short = 't')]
        file: Option<String>,
        #[arg(
            long = "keys",
            short = 'k',
            value_delimiter = ',',
            num_args = 1..,
        )]
        keys: Option<Vec<String>>,
    },

    #[command(name = "version", about = "Print the version")]
    Version {
        #[arg(long = "verbose", short = 'v')]
        verbose: bool,
    },
}

#[derive(clap::Subcommand, Debug)]
pub enum ProfileCommand {
    #[command(
        name = "create",
        about = "Create a new profile",
        override_usage = "envio profile create <PROFILE_NAME> [OPTIONS]"
    )]
    Create {
        #[arg(required = true)]
        profile_name: String,
        #[arg(long = "from-file", short = 'f')]
        envs_file: Option<String>,
        #[arg(
            long = "envs",
            short = 'e',
            value_delimiter = ' ',
            num_args = 1..,
        )]
        envs: Option<Vec<String>>,
        #[arg(long = "gpg-key", short = 'g')]
        gpg: Option<String>,
        #[arg(long = "add-comments", short = 'c')]
        add_comments: bool,
        #[arg(long = "add-expiration-date", short = 'x')]
        add_expiration_date: bool,
    },

    #[command(
        name = "delete",
        about = "Delete a profile",
        override_usage = "envio profile delete <PROFILE_NAME>"
    )]
    Delete {
        #[arg(required = true)]
        profile_name: String,
    },

    #[command(
        name = "list",
        about = "List all profiles",
        override_usage = "envio profile list [OPTIONS]"
    )]
    List {
        #[arg(long = "no-pretty-print", short = 'v')]
        no_pretty_print: bool,
    },

    #[command(
        name = "show",
        about = "Show environment variables in a profile",
        override_usage = "envio profile show <PROFILE_NAME> [OPTIONS]"
    )]
    Show {
        #[arg(required = true)]
        profile_name: String,
        #[arg(long = "no-pretty-print", short = 'v')]
        no_pretty_print: bool,
        #[arg(long = "display-comments", short = 'c')]
        display_comments: bool,
        #[arg(long = "display-expiration-date", short = 'x')]
        display_expiration_date: bool,
    },
}
