use clap::Parser;

#[derive(Parser, Debug, Clone)]
/// IC3
pub struct Options {
    /// input aiger file
    pub model: String,

    /// verbose level
    #[arg(short, default_value_t = 1)]
    pub verbose: usize,

    /// ctg
    #[arg(long, default_value_t = false)]
    pub ctg: bool,

    /// random seed
    #[arg(short, long)]
    pub random: Option<usize>,

    /// print witness
    #[arg(long, default_value_t = false)]
    pub witness: bool,

    /// verify
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// verify by certifaiger
    #[arg(long, default_value_t = false, requires("verify"))]
    pub certifaiger: bool,

    /// save frames
    #[arg(long, default_value_t = false)]
    pub save_frames: bool,

    /// bmc engine
    #[arg(long, default_value_t = false)]
    pub bmc: bool,

    /// k-induction engine
    #[arg(long, default_value_t = false)]
    pub kind: bool,

    /// imc engine
    #[arg(long, default_value_t = false)]
    pub imc: bool,

    /// portfolio
    #[arg(short, long, default_value_t = false)]
    pub portfolio: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options::parse_from([""])
    }
}
