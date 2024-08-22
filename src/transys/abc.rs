use abc::Abc;
use aig::Aig;
use std::{mem::take, time::Duration};

fn preprocess(f: String) {
    let mut aig = Aig::from_file(&f);
    let num_input = aig.inputs.len();
    let num_latchs = aig.latchs.len();
    let num_constraints = aig.constraints.len();
    if aig.outputs.is_empty() {
        aig.outputs.push(aig.bads[0]);
        aig.bads.clear();
    }
    let latchs = take(&mut aig.latchs);
    for l in latchs.iter() {
        aig.inputs.push(l.input);
        aig.outputs.push(l.next);
    }
    for c in take(&mut aig.constraints) {
        aig.outputs.push(c);
    }
    assert!(aig.inputs.len() == num_input + num_latchs);
    assert!(aig.outputs.len() == 1 + num_latchs + num_constraints);
    let mut abc = Abc::new();
    abc.read_aig(&aig);
    drop(aig);
    abc.execute_command("&get; &fraig -y -r; &put; rewrite; refactor");
    let mut abc_aig = abc.write_aig();
    for i in 0..num_latchs {
        let mut l = latchs[i];
        l.input = abc_aig.inputs[num_input + i];
        l.next = abc_aig.outputs[1 + i];
        abc_aig.latchs.push(l);
    }
    abc_aig.inputs.truncate(num_input);
    for i in 0..num_constraints {
        abc_aig
            .constraints
            .push(abc_aig.outputs[1 + num_latchs + i]);
    }
    abc_aig.outputs.truncate(1);

    assert!(abc_aig.inputs.len() == num_input);
    assert!(abc_aig.latchs.len() == num_latchs);
    assert!(abc_aig.constraints.len() == num_constraints);

    abc_aig.to_file(&f, false);
}

pub fn abc_preprocess(mut aig: Aig) -> Aig {
    let tmpfile = tempfile::NamedTempFile::new().unwrap();
    let path = tmpfile.path().as_os_str().to_str().unwrap();
    aig.to_file(path, false);
    let mut join = procspawn::spawn(path.to_string(), preprocess);
    if join.join_timeout(Duration::from_secs(50)).is_ok() {
        aig = Aig::from_file(path);
    } else {
        println!("abc preprocess timeout");
        let _ = join.kill();
    }
    drop(tmpfile);
    aig
}
