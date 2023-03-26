use clap::Args;
use clap::Parser;
//use crate::commands::Command;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
pub struct CommandArgs {
    pub args: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    #[clap(name = "create", about = "Create a new profile")]
    Create(CommandArgs),
    #[clap(name = "add", about = "Add a new environment variable to a profile")]
    Add(CommandArgs),
    #[clap(name = "load", about = "Load a profile in the current session")]
    Load(CommandArgs),
    #[clap(name = "unload", about = "Unload a profile from the current session")]
    Unload(CommandArgs),
    #[clap(name = "launch", about = "Launch a program with a profile")]
    Launch(CommandArgs),
    #[clap(
        name = "remove",
        about = "Remove a environment variable from a profile"
    )]
    Remove(CommandArgs),
    #[clap(
        name = "list",
        about = "List all the environment variables in a profile or all the profiles"
    )]
    List(CommandArgs),
    #[clap(name = "update", about = "Update a environment variable in a profile")]
    Update(CommandArgs),
    #[clap(
        name = "export",
        about = "Export a profile to a file if no file is specified it will be exported to a file named .env"
    )]
    Export(CommandArgs),
    #[clap(name = "import", about = "Import a profile from a file")]
    Import(CommandArgs),
    #[clap(name = "version", about = "Print the version")]
    Version(CommandArgs),
}
