use logic_form::Cube;

pub fn cube_subsume(x: &Cube, y: &Cube) -> bool {
    if x.len() > y.len() {
        return false;
    }
    let mut j = 0;
    for i in 0..x.len() {
        while j < y.len() && x[i].var() > y[j].var() {
            j += 1;
        }
        if j == y.len() || x[i] != y[j] {
            return false;
        }
    }
    true
}

pub fn cube_subsume_init(x: &Cube) -> bool {
    for i in 0..x.len() {
        if !x[i].compl() {
            return false;
        }
    }
    true
}
