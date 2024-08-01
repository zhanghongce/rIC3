use clap::{ArgGroup, Args, Parser};

/// rIC3 model checker
#[derive(Parser, Debug, Clone)]
#[command(version, about)]
#[command(group = ArgGroup::new("engine").required(true).multiple(false))]
pub struct Options {
    /// model file in aiger format
    pub model: String,

    /// ic3 engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub ic3: bool,

    #[command(flatten)]
    pub ic3_options: IC3Options,

    /// bmc engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub bmc: bool,

    #[command(flatten)]
    pub bmc_options: BMCOptions,

    /// k-induction engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub kind: bool,

    #[command(flatten)]
    pub kind_options: KindOptions,

    /// imc engine
    #[arg(long, default_value_t = false, group = "engine")]
    pub imc: bool,

    /// portfolio
    #[arg(long, default_value_t = false, group = "engine")]
    pub portfolio: bool,

    #[command(flatten)]
    pub preprocess: PreprocessOptions,

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

#[derive(Args, Clone, Debug)]
pub struct IC3Options {
    /// ic3 counterexample to generalization
    #[arg(long = "ic3-ctg", default_value_t = false, requires = "ic3")]
    pub ctg: bool,

    /// ic3 xor generalization
    #[arg(long = "ic3-xor", default_value_t = false, requires = "ic3")]
    pub xor: bool,

    /// ic3 counterexample to propagation
    #[arg(long = "ic3-ctp", default_value_t = false, requires = "ic3")]
    pub ctp: bool,
}

#[derive(Args, Clone, Debug)]
pub struct BMCOptions {
    /// use kissat solver, otherwise cadical
    #[arg(long = "bmc-kissat", default_value_t = false, requires = "bmc")]
    pub kissat: bool,
}

#[derive(Args, Clone, Debug)]
pub struct KindOptions {
    /// no bmc check in kind
    #[arg(long = "kind-no-bmc", default_value_t = false, requires = "kind")]
    pub no_bmc: bool,
}

#[derive(Args, Clone, Debug)]
pub struct PreprocessOptions {
    /// sec preprocess
    #[arg(long = "sec", default_value_t = false)]
    pub sec: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options::parse_from([""])
    }
}
