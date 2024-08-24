use crate::{Engine, Options};
use aig::Aig;
use process_control::{ChildExt, Control};
use std::{
    env::current_exe,
    fs::File,
    io::Read,
    mem::take,
    process::{Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread::spawn,
};
use tempfile::NamedTempFile;

pub struct Portfolio {
    option: Options,
    engines: Vec<Command>,
    certify_file: Option<NamedTempFile>,
}

impl Portfolio {
    pub fn new(option: Options) -> Self {
        let mut engines = Vec::new();
        let mut new_engine = |args: &[&str]| {
            let mut engine = Command::new(current_exe().unwrap());
            engine.arg(&option.model);
            engine.args(["-v", "0", "--not-certify"]);
            engine.args(args);
            engines.push(engine);
        };
        new_engine(&["--ic3"]);
        new_engine(&["--ic3", "--ic3-ctg"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-abs-cst"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-ctp"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-inn"]);
        new_engine(&["--ic3", "--ic3-ctg", "--ic3-ctp", "--ic3-inn"]);
        // new_engine(&["--ic3", "--ic3-bwd", "--ic3-ctg"]);

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
        // new_engine(&["--kind", "--step", "1"]);
        Self {
            option,
            engines,
            certify_file: None,
        }
    }
}

impl Engine for Portfolio {
    fn check(&mut self) -> Option<bool> {
        let mut engines = Vec::new();
        let result = Arc::new((Mutex::new(None), Condvar::new()));
        let lock = result.0.lock().unwrap();
        for mut engine in take(&mut self.engines) {
            let certify_file = if self.option.certify_path.is_some() || !self.option.not_certify {
                let certify_file = tempfile::NamedTempFile::new().unwrap();
                let certify_path = certify_file.path().as_os_str().to_str().unwrap();
                engine.arg(&certify_path);
                Some(certify_file)
            } else {
                None
            };
            let mut child = engine.stderr(Stdio::piped()).spawn().unwrap();
            engines.push(child.id() as i32);
            let option = self.option.clone();
            let result = result.clone();
            spawn(move || {
                let config = engine
                    .get_args()
                    .skip(4)
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
                    *lock = Some((res, config, certify_file));
                    result.1.notify_one();
                }
            });
        }
        let mut result = result.1.wait(lock).unwrap();
        let result = take(&mut *result);
        let (res, config, certify_file) = result.unwrap();
        self.certify_file = certify_file;
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
        Some(res)
    }

    fn certifaiger(&mut self, _aig: &aig::Aig) -> Aig {
        Aig::from_file(
            self.certify_file
                .as_ref()
                .unwrap()
                .path()
                .as_os_str()
                .to_str()
                .unwrap(),
        )
    }

    fn witness(&mut self, _aig: &Aig) -> String {
        let mut res = String::new();
        File::open(
            self.certify_file
                .as_ref()
                .unwrap()
                .path()
                .as_os_str()
                .to_str()
                .unwrap(),
        )
        .unwrap()
        .read_to_string(&mut res)
        .unwrap();
        res
    }
}
