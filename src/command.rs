use clap::Parser;

#[derive(Parser, Debug, Clone, Copy)]
/// Parallel IC3
pub struct Args {
    /// verbose
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// parallel
    #[arg(short, long, default_value_t = 1)]
    pub parallel: usize,
}
