use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Read, Write},
};

use crate::Ic3;
use cudd::Cudd;
use logic_form::{Cube, Var};

impl Ic3 {
    pub fn print_frames(&mut self) {
        todo!();
        let json = serde_json::to_string(&self.frames.frames).unwrap();
        let mut file = File::create("frames.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}

pub fn affinity(x: &Cube, y: &Cube) -> (f64, Cube) {
    let xs: HashSet<Var> = x.iter().map(|l| l.var()).collect();
    let ys: HashSet<Var> = y.iter().map(|l| l.var()).collect();
    let common: Vec<Var> = xs.intersection(&ys).copied().collect();
    (
        common.len() as f64 / (xs.len() + ys.len() - common.len()) as f64,
        // x.intersection(y),
        Cube::new(),
    )
}

pub fn print_affinity(inv: &[Cube]) {
    let mut affinity_matrix = vec![vec![0_f64; inv.len()]; inv.len()];
    for i in 0..inv.len() {
        dbg!(i);
        affinity_matrix[i][i] = 1.0;
        for j in i + 1..inv.len() {
            let (af, _) = affinity(&inv[i], &inv[j]);
            affinity_matrix[i][j] = af;
            affinity_matrix[j][i] = af;
            // if af > 0.8 {
            //     dbg!(af);
            //     let ci: Cube = inv[i]
            //         .iter()
            //         .filter(|l| !common.contains(l))
            //         .copied()
            //         .collect();
            //     let cj: Cube = inv[j]
            //         .iter()
            //         .filter(|l| !common.contains(l))
            //         .copied()
            //         .collect();
            //     println!("{:?}", common);
            //     println!("{:?}", ci);
            //     println!("{:?}", cj);
            // }
        }
    }
    // find_clique(inv, &affinity_matrix);
}

// pub fn cudd_test(inv: &[Cube], clique: &[usize]) {
//     let mut freq = HashMap::new();
//     for c in clique {
//         for l in inv[*c].iter() {
//             *freq.entry(l.var()).or_insert(0) += 1;
//         }
//     }
//     let mut sorted_vec: Vec<(&Var, &i32)> = freq.iter().collect();
//     sorted_vec.sort_by(|a, b| b.1.cmp(a.1));
//     let mut var_map = HashMap::new();
//     for i in 0..sorted_vec.len() {
//         var_map.insert(*sorted_vec[i].0, i);
//     }
//     let cudd = Cudd::new();
//     let mut clq = cudd.constant(false);
//     for c in clique {
//         let mut c_bdd = cudd.constant(true);
//         for l in inv[*c].iter() {
//             let var = *var_map.get(&l.var()).unwrap();
//             let mut l_bdd = cudd.ith_var(var);
//             if !l.polarity() {
//                 l_bdd = !l_bdd;
//             }
//             c_bdd &= l_bdd;
//         }
//         clq |= c_bdd;
//     }
//     dbg!(clq.size());
// }

// pub fn find_clique(inv: &[Cube], v: &Vec<Vec<f64>>) {
//     let len = v.len();
//     let mut finded = HashSet::new();
//     for i in 0..len {
//         if finded.contains(&i) {
//             continue;
//         }
//         finded.insert(i);
//         let mut clique = vec![i];
//         for j in i + 1..len {
//             if clique.iter().all(|x| v[*x][j] > 0.5) {
//                 clique.push(j);
//                 finded.insert(j);
//             }
//         }
//         if clique.len() > 1 {
//             println!("{:?}", clique);
//             for c in clique.iter() {
//                 println!("{:?}", inv[*c]);
//             }
//             // cudd_test(inv, &clique);
//         }
//     }
// }

pub fn filter(inv: Vec<Cube>, f: &[Var]) -> Vec<Cube> {
    let mut ans = Vec::new();
    for c in inv {
        let mut sat = true;
        let mut a = Cube::new();
        for fv in f.iter() {
            let fl = fv.lit();
            if c.contains(&fl) {
                a.push(fl);
            } else if c.contains(&!fl) {
                a.push(!fl);
            } else {
                sat = false;
                break;
            }
        }
        if !sat {
            continue;
        }
        for cl in c.iter() {
            if !f.contains(&cl.var()) {
                a.push(*cl);
            }
        }
        ans.push(a);
    }
    ans
}

#[test]
pub fn analysis() {
    let mut file = File::open("frames.json").expect("Failed to open file");
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();
    let mut frames: Vec<Vec<Cube>> = serde_json::from_str(&json).unwrap();
    for i in 1..frames.len() {
        println!("frame {}", i);
        frames[i].sort();
        for c in frames[i].iter() {
            println!("{:?}", c);
        }
    }

    // let mut freq = HashMap::new();
    // let f = [Var::new(977), Var::new(490)];
    // inveriant = filter(inveriant, &f);
    // inveriant.sort();
    // print_affinity(frames.last().unwrap());
    // dbg!(inveriant.len());
    // for mut c in inveriant {
    //     for l in c.iter() {
    //         *freq.entry(l.var()).or_insert(0) += 1;
    //     }
    //     // let pos = c
    //     //     .iter()
    //     //     .position(|l| *l == Lit::new(Var::new(621), true))
    //     //     .unwrap();
    //     // c.remove(pos);
    //     println!("{:?}", c);
    // }
    // dbg!("freq");
    // let mut sorted_vec: Vec<(&Var, &i32)> = freq.iter().collect();
    // sorted_vec.sort_by(|a, b| b.1.cmp(a.1));
    // for (key, value) in sorted_vec {
    //     println!("var: {}, num: {}", key, value);
    // }
}
