use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about)]
pub struct CLIArgs {
    /// Path to the engine executable
    #[clap(short = 'P', long)]
    pub engine_path: Option<String>,

    /// Tickrate in milliseconds
    #[clap(short = 'T', long, default_value = "200")]
    pub tickrate: u64,
}
