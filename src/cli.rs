use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "emd")]
#[command(about = "AWS resource explorer and Markdown documentation generator")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Update to the latest version
    Update,
}

pub fn run() -> Option<()> {
    let cli = Cli::parse();

    match cli.command? {
        Command::Update => {
            if let Err(e) = crate::update::perform_update() {
                eprintln!("Update failed: {}", e);
                std::process::exit(1);
            }
            Some(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parse_without_subcommand() {
        let cli = Cli::parse_from(["emd"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parse_update_subcommand() {
        let cli = Cli::parse_from(["emd", "update"]);
        assert!(matches!(cli.command, Some(Command::Update)));
    }
}
