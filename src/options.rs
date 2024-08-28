use clap::{ArgGroup, Args, Parser, ValueEnum};

/// rIC3 model checker
#[derive(Parser, Debug, Clone)]
#[command(version, about)]
#[command(group = ArgGroup::new("engine").required(false).multiple(false))]
pub struct Options {
    /// model checking engine
    #[arg(short, long, value_enum, default_value_t = Engine::Portfolio)]
    pub engine: Engine,

    /// model file in aiger format or in btor2 format.
    /// for btor model, the file name should be suffixed with .btor or .btor2
    pub model: String,

    /// verify
    #[arg(long, default_value_t = false)]
    pub certify: bool,

    /// certifaiger or witness output path
    pub certify_path: Option<String>,

    #[command(flatten)]
    pub ic3_options: IC3Options,

    #[command(flatten)]
    pub bmc_options: BMCOptions,

    #[command(flatten)]
    pub kind_options: KindOptions,

    #[command(flatten)]
    pub preprocess: PreprocessOptions,

    /// step length
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub step: u32,

    /// random seed
    #[arg(long, default_value_t = 0)]
    pub rseed: u64,

    /// verbose level
    #[arg(short, default_value_t = 1)]
    pub verbose: usize,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Engine {
    /// ic3
    IC3,
    /// general ic3
    GIC3,
    /// k-induction
    Kind,
    /// bmc
    BMC,
    /// portfolio
    Portfolio,
}

#[derive(Args, Clone, Debug)]
pub struct IC3Options {
    /// ic3 counterexample to generalization
    #[arg(long = "ic3-ctg", default_value_t = false, requires = "ic3")]
    pub ctg: bool,

    /// max number of ctg
    #[arg(long = "ic3-ctg-max", default_value_t = 3, requires = "ic3")]
    pub ctg_max: usize,

    /// ctg limit
    #[arg(long = "ic3-ctg-limit", default_value_t = 5, requires = "ic3")]
    pub ctg_limit: usize,

    /// ic3 xor generalization
    #[arg(long = "ic3-xor", default_value_t = false, requires = "ic3")]
    pub xor: bool,

    /// ic3 counterexample to propagation
    #[arg(long = "ic3-ctp", default_value_t = false, requires = "ic3")]
    pub ctp: bool,

    /// ic3 with internal signals (FMCAD'21)
    #[arg(long = "ic3-inn", default_value_t = false, requires = "ic3")]
    pub inn: bool,

    /// ic3 with backward
    #[arg(long = "ic3-bwd", default_value_t = false, requires = "ic3")]
    pub bwd: bool,

    /// ic3 with abstract constrains
    #[arg(long = "ic3-abs-cst", default_value_t = false, requires = "ic3")]
    pub abs_cst: bool,
}

#[derive(Args, Clone, Debug)]
pub struct BMCOptions {
    /// bmc single step time limit
    #[arg(long = "bmc-time-limit", requires = "bmc")]
    pub time_limit: Option<u64>,
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
