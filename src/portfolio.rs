use crate::Options;
use process_control::{ChildExt, Control};
use std::{
    env::current_exe,
    process::{Command, Stdio},
    sync::mpsc::channel,
    thread::spawn,
};

pub struct Portfolio {
    args: Options,
}

impl Portfolio {
    pub fn new(args: Options) -> Self {
        Self { args }
    }

    pub fn check(&mut self) -> bool {
        let mut engines = Vec::new();
        let mut engine = Command::new(current_exe().unwrap());
        engine.arg(&self.args.model);
        engines.push(engine);

        let (tx, rx) = channel::<(String, bool)>();
        for mut engine in engines {
            let tx = tx.clone();
            spawn(move || {
                let mut child = engine.stderr(Stdio::piped()).spawn().unwrap();
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
                    let config = engine.get_args();
                    let config = config
                        .map(|cstr| cstr.to_str().unwrap())
                        .collect::<Vec<&str>>()
                        .join(" ");
                    let _ = tx.send((config, res));
                } else {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(child.id() as i32),
                        nix::sys::signal::Signal::SIGKILL,
                    )
                    .unwrap();
                };
            });
        }
        let (config, res) = rx.recv().unwrap();
        println!("best configuration: {config}");
        res
    }
}
