use crate::Options;
use process_control::{ChildExt, Control};
use std::{
    env::current_exe,
    mem::take,
    process::{Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread::spawn,
};

pub struct Portfolio {
    option: Options,
    engines: Vec<Command>,
}

impl Portfolio {
    pub fn new(option: Options) -> Self {
        let mut engines = Vec::new();
        let mut new_engine = |args: &[&str]| {
            let mut engine = Command::new(current_exe().unwrap());
            engine.args(["-v", "0"]);
            engine.arg(&option.model);
            engine.args(args);
            engines.push(engine);
        };
        new_engine(&["--ic3"]);
        new_engine(&["--ic3", "--ic3-ctg"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-ctp"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-inn"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-ctp", "--ic3-inn"]);
        new_engine(&["--ic3", "--ic3-abs-cst", "--ic3-ctg", "--ic3-inn"]);
        new_engine(&["--ic3", "--ic3-bwd", "--ic3-ctg"]);

        new_engine(&["--bmc", "--step", "10"]);
        new_engine(&["--bmc", "--bmc-kissat", "--step", "70"]);
        new_engine(&["--bmc", "--bmc-kissat", "--step", "135"]);
        new_engine(&[
            "--bmc",
            "--bmc-kissat",
            "--bmc-time-limit",
            "100",
            "--step",
            "100",
        ]);
        new_engine(&["--kind", "--step", "1"]);
        Self { option, engines }
    }

    pub fn check(&mut self) -> Option<bool> {
        let mut engines = Vec::new();
        let result = Arc::new((Mutex::new(None), Condvar::new()));
        let lock = result.0.lock().unwrap();
        for mut engine in take(&mut self.engines) {
            let mut child = engine.stderr(Stdio::piped()).spawn().unwrap();
            engines.push(child.id() as i32);
            let option = self.option.clone();
            let result = result.clone();
            spawn(move || {
                let config = engine
                    .get_args()
                    .skip(2)
                    .map(|cstr| cstr.to_str().unwrap())
                    .collect::<Vec<&str>>()
                    .join(" ");
                if option.verbose > 1 {
                    println!("start engine: {config}");
                }
                let status = child
                    .controlled()
                    .memory_limit(1024 * 1024 * 1024 * 16)
                    .wait()
                    .unwrap()
                    .unwrap();
                let res = match status.code() {
                    Some(10) => false,
                    Some(20) => true,
                    e => {
                        if option.verbose > 0 && result.0.lock().unwrap().is_none() {
                            println!("{config} unsuccessfully exited, exit code: {:?}", e);
                        }
                        return;
                    }
                };
                let mut lock = result.0.lock().unwrap();
                if lock.is_none() {
                    *lock = Some((res, config));
                    result.1.notify_one();
                }
            });
        }
        let result = result.1.wait(lock).unwrap();
        let (res, config) = result.as_ref().unwrap();
        println!("best configuration: {}", config);
        let mut cmd = "(".to_string();
        for (i, pid) in engines.into_iter().enumerate() {
            if i != 0 {
                cmd.push_str(" && ");
            }
            cmd.push_str(&format!(r#"(pstree -p {})"#, pid));
        }
        cmd.push_str(&format!(
            r#") | grep -oP '\(\K\d+' | sort -u | xargs -n 1 kill -9"#
        ));
        Command::new("sh").args(["-c", &cmd]).output().unwrap();
        Some(*res)
    }
}
