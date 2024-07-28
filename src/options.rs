use clap::{ArgGroup, Parser};

/// rIC3 model checker
#[derive(Parser, Debug, Clone)]
#[command(version, about)]
#[command(group = ArgGroup::new("engine").required(false).multiple(false))]
pub struct Options {
    /// model file in aiger format
    pub model: String,

    /// ic3 engine
    #[arg(long, default_value_t = true, group = "engine")]
    pub ic3: bool,

    /// ic3 ctg
    #[arg(long, default_value_t = false, requires = "ic3")]
    pub ctg: bool,

    /// bmc engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub bmc: bool,

    /// k-induction engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub kind: bool,

    /// imc engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub imc: bool,

    /// portfolio
    #[arg(long, default_value_t = false, group = "engine")]
    pub portfolio: bool,

    /// step length
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub step: u32,

    /// random seed
    #[arg(long, default_value_t = 5)]
    pub rseed: usize,

    /// print witness
    #[arg(long, default_value_t = false)]
    pub witness: bool,

    /// verify
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// verify by certifaiger
    #[arg(long, default_value_t = false, requires = "verify")]
    pub certifaiger: bool,

    /// verbose level
    #[arg(short, default_value_t = 1)]
    pub verbose: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options::parse_from([""])
    }
}
