use crate::Options;
use process_control::{ChildExt, Control};
use std::{
    env::current_exe,
    mem::take,
    process::{Command, Stdio},
    sync::mpsc::channel,
    thread::spawn,
};

pub struct Portfolio {
    _option: Options,
    engines: Vec<Command>,
}

impl Portfolio {
    pub fn new(option: Options) -> Self {
        let mut engines = Vec::new();
        let mut new_engine = |args: &[&str]| {
            let mut engine = Command::new(current_exe().unwrap());
            engine.arg(&option.model);
            engine.args(&["-v", "0"]);
            engine.args(args);
            engines.push(engine);
        };
        // ic3
        new_engine(&["--ic3"]);
        // bmc kissat step 70
        new_engine(&["--bmc", "--kissat", "--step", "70"]);
        // bmc kissat step 130
        new_engine(&["--bmc", "--kissat", "--step", "130"]);
        // kind
        new_engine(&["--kind"]);
        Self {
            _option: option,
            engines,
        }
    }

    pub fn check(&mut self) -> bool {
        let (tx, rx) = channel::<(String, bool)>();
        let mut engines = Vec::new();
        for mut engine in take(&mut self.engines) {
            let mut child = engine.stderr(Stdio::piped()).spawn().unwrap();
            engines.push(child.id() as i32);
            let tx = tx.clone();
            spawn(move || {
                let config = engine
                    .get_args()
                    .map(|cstr| cstr.to_str().unwrap())
                    .collect::<Vec<&str>>()
                    .join(" ");
                // println!("start engine: {config}");
                let output = child
                    .controlled()
                    .memory_limit(1024 * 1024 * 1024 * 16)
                    .wait()
                    .unwrap();
                if let Some(status) = output {
                    let res = match status.code() {
                        Some(10) => false,
                        Some(20) => true,
                        _ => return,
                    };
                    let _ = tx.send((config, res));
                } else {
                    let _ = nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(child.id() as i32),
                        nix::sys::signal::Signal::SIGKILL,
                    );
                };
            });
        }
        let (config, res) = rx.recv().unwrap();
        println!("best configuration: {config}");
        for pid in engines {
            let _ = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGKILL,
            );
        }
        res
    }
}
