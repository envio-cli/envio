use clap::Parser;

#[derive(Parser, Debug)]
#[clap(disable_help_subcommand = true)]
pub struct ClapApp {
    #[command(subcommand)]
    pub command: Command,
    #[arg(
        long = "diagnostic",
        help = "Show diagnostic information for bug reports",
        global = true
    )]
    pub diagnostic: bool,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[command(
        name = "create",
        about = "Create a new profile",
        visible_aliases = &["new"],
        override_usage = "envio create <PROFILE_NAME> [OPTIONS]"
    )]
    Create {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(
            long = "description",
            short = 'd',
            help = "optional note or description of the profile"
        )]
        description: Option<String>,
        #[arg(
            long = "from-file",
            short = 'f',
            help = "file path to load environment variables from"
        )]
        envs_file: Option<String>,
        #[arg(
            long = "envs",
            short = 'e',
            num_args = 1..,
            help = "environment variables to add (format: KEY=VALUE or only provide KEY and the value will be prompted for)"
        )]
        envs: Option<Vec<String>>,
        #[arg(long = "cipher-kind", short = 'k', help = "encryption cipher to use")]
        cipher_kind: Option<String>,
        #[arg(
            long = "comments",
            short = 'c',
            help = "add comments to the provided environment variables"
        )]
        comments: bool,
        #[arg(
            long = "expires",
            short = 'x',
            help = "add expiration dates to the provided environment variables"
        )]
        expires: bool,
    },

    #[command(
        name = "delete",
        about = "Delete a profile",
        visible_aliases = &["remove"],
        override_usage = "envio delete <PROFILE_NAME>"
    )]
    Delete {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
    },

    #[command(
        name = "list",
        about = "List all profiles",
        visible_aliases = &["ls"],
        override_usage = "envio list [OPTIONS]"
    )]
    List {
        #[arg(long = "no-pretty-print", help = "disable pretty printing")]
        no_pretty_print: bool,
    },

    #[command(
        name = "show",
        about = "Show environment variables in a profile",
        override_usage = "envio show <PROFILE_NAME> [OPTIONS]"
    )]
    Show {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(long = "show-comments", short = 'c', help = "display comments")]
        show_comments: bool,
        #[arg(
            long = "show-expiration",
            short = 'x',
            help = "display expiration dates"
        )]
        show_expiration: bool,
        #[arg(long = "no-pretty-print", help = "disable pretty printing")]
        no_pretty_print: bool,
    },

    #[command(
        name = "set",
        about = "Set or update environment variables in a profile",
        override_usage = "envio set <PROFILE_NAME> <ENVS>... [OPTIONS]"
    )]
    Set {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(required = true, num_args = 1.., help = "environment variables to set (format: KEY=VALUE or only provide KEY and the value will be prompted for)")]
        envs: Vec<String>,
        #[arg(
            long = "comments",
            short = 'c',
            help = "add comments to the provided environment variables"
        )]
        comments: bool,
        #[arg(
            long = "expires",
            short = 'x',
            help = "add expiration dates to the provided environment variables"
        )]
        expires: bool,
    },

    #[command(
        name = "unset",
        about = "Remove environment variables from a profile",
        override_usage = "envio unset <PROFILE_NAME> <KEYS>... [OPTIONS]"
    )]
    Unset {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(required = true, value_delimiter = ' ', num_args = 1.., help = "keys of environment variables to remove")]
        keys: Vec<String>,
    },

    #[command(
        name = "load",
        about = "Load environment variables from a profile for use in terminal sessions",
        override_usage = "envio load <PROFILE_NAME>"
    )]
    Load {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
    },

    #[cfg(target_family = "unix")]
    #[command(
        name = "unload",
        about = "Unload previously loaded environment variables from terminal sessions",
        override_usage = "envio unload"
    )]
    Unload,

    #[cfg(target_family = "windows")]
    #[command(
        name = "unload",
        about = "Unload previously loaded environment variables from terminal sessions",
        override_usage = "envio unload <PROFILE_NAME>"
    )]
    Unload {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
    },

    #[command(
        name = "run",
        about = "Run a command using environment variables from a profile",
        override_usage = "envio run <PROFILE_NAME> -- <COMMAND>"
    )]
    Run {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(last = true, required = true, help = "command to run")]
        command: Vec<String>,
    },

    #[command(
        name = "import",
        about = "Import a profile from a file or url",
        override_usage = "envio import <SOURCE> [OPTIONS]"
    )]
    Import {
        #[arg(required = true, help = "source file or url")]
        source: String,
        #[arg(
            long = "profile-name",
            short = 'n',
            help = "name for the imported profile"
        )]
        profile_name: Option<String>,
    },

    #[command(
        name = "export",
        about = "Export the environment variables of a profile to a file",
        override_usage = "envio export <PROFILE_NAME> [OPTIONS]"
    )]
    Export {
        #[arg(required = true, help = "name of the profile")]
        profile_name: String,
        #[arg(
            long = "output-file-path",
            short = 'o',
            help = "output file path (default: .env)"
        )]
        output_file_path: Option<String>,
        #[arg(
            long = "keys",
            short = 'k',
            value_delimiter = ',',
            num_args = 1..,
            help = "comma-separated list of keys to export (type 'select' to choose interactively)"
        )]
        keys: Option<Vec<String>>,
    },

    #[command(
        name = "tui",
        about = "Launch the interactive TUI application",
        override_usage = "envio tui"
    )]
    Tui,

    #[command(
        name = "completion",
        about = "Show shell completion for the provided shell",
        override_usage = "envio completion <SHELL>"
    )]
    Completion {
        #[arg(
            required = true,
            help = "shell to show completion for (bash, zsh, fish, powershell)",
            value_parser = clap::builder::PossibleValuesParser::new(&["bash", "zsh", "fish", "powershell"])
        )]
        shell: String,
    },

    #[command(
        name = "version",
        about = "Print version information",
        override_usage = "envio version [OPTIONS]"
    )]
    Version {
        #[arg(
            long = "verbose",
            short = 'v',
            help = "show verbose version information"
        )]
        verbose: bool,
    },
}
