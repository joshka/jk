use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "jk", version, about = "Log-first jj TUI")]
pub struct Cli {
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,
}
