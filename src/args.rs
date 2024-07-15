use clap::Parser;

#[derive(Parser, Debug, Clone)]
/// IC3
pub struct Args {
    /// input aiger file
    pub model: String,

    /// verbose
    #[arg(short, default_value_t = false)]
    pub verbose: bool,

    /// verbose all
    #[arg(short = 'V', default_value_t = false, requires("verbose"))]
    pub verbose_all: bool,

    /// random seed
    #[arg(short, long)]
    pub random: Option<usize>,

    /// print witness
    #[arg(long, default_value_t = false)]
    pub witness: bool,

    /// verify
    #[arg(long, default_value_t = true)]
    pub verify: bool,

    /// save frames
    #[arg(long, default_value_t = false)]
    pub save_frames: bool,

    /// bmc engine
    #[arg(long, default_value_t = false)]
    pub bmc: bool,

    /// k-induction engine
    #[arg(long, default_value_t = false)]
    pub kind: bool,

    /// portfolio
    #[arg(short, long, default_value_t = false)]
    pub portfolio: bool,
}

impl Default for Args {
    fn default() -> Self {
        Args::parse_from([""])
    }
}
