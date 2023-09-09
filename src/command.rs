use clap::Parser;

#[derive(Parser, Debug, Clone)]
/// Parallel IC3
pub struct Args {
    /// input aiger file
    pub model: Option<String>,
    /// verbose
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// parallel
    #[arg(short, long, default_value_t = 1)]
    pub parallel: usize,

    /// counter example to propagate
    #[arg(long, default_value_t = false)]
    pub ctp: bool,
}
