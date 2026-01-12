use clap::Parser;

/// Detects cargo dependency conflicts based on a ruleset.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to a file that declares the ruleset.
    /// Defaults to the `conflict.toml` file on the root workspace.
    #[arg(long = "ruleset-file", short = 'f')]
    pub ruleset_file: Option<String>,

    /// Directory of a workspace that contains the `Cargo.toml` file.
    /// Defaults to the current workspace.
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,
}
