use clap::Parser;

#[derive(Parser, Debug, Clone)]
/// IC3
pub struct Args {
    /// input aiger file
    pub model: Option<String>,
    /// verbose
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// parallel
    #[arg(short, long, default_value_t = 1)]
    pub parallel: usize,
    /// counter example to generalize
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub ctg: bool,
    /// cav23
    #[arg(long, default_value_t = false)]
    pub cav23: bool,
}
