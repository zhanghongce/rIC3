use clap::Parser;

#[derive(Parser, Debug)]
/// Hardware Model Checking
pub struct Args {
    /// verbose
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}
