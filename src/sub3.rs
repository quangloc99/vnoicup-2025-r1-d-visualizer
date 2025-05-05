mod logic;
use logic::cell::Cell;
use logic::solver;
use std::io::*;

fn main() {
    let stdout = std::io::stdout();
    let mut writer = std::io::BufWriter::new(stdout.lock());
    let lines = std::io::stdin().lines().map(|x| x.unwrap());
    let mut tokens = lines.flat_map(|x| x.split_ascii_whitespace().map(|x| x.to_string()).collect::<Vec<_>>());
    let mut next_usize = move || tokens.next().unwrap().parse::<usize>().ok().expect("Failed to parse");

    let k = next_usize();
    let q = next_usize();

    let mut cur_empty = Cell(0, 0);

    for _qid in 0..q {
        let r = next_usize();
        let c = next_usize();
        let target = Cell(r, c);

        let ans = solver::dist(k, cur_empty, target);

        writeln!(writer, "{ans}").ok();
        cur_empty = target;
    }
}
