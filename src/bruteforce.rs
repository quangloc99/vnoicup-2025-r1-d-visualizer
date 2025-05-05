#![allow(unexpected_cfgs, unused_macros)]

mod logic;
use logic::board::{build_move_graph, Board};
use logic::cell::Cell;
use {std::collections::*, std::io, std::io::*};

macro_rules! eprintln { ($($arg:tt)*) => { if cfg!(LOCAL) { std::eprintln!($($arg)*); } }; }
macro_rules! dbg { ($($arg:expr),*) => { eprintln!(concat!($("[", stringify!($arg), " = {:?}] "),*) $(, $arg)*) }}

fn main() {
    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout.lock());
    let lines = io::stdin().lines().map(|x| x.unwrap());
    let mut tokens = lines.flat_map(|x| x.split_ascii_whitespace().map(|x| x.to_string()).collect::<Vec<_>>());
    let mut next_usize = move || tokens.next().unwrap().parse::<usize>().ok().expect("Failed to parse");

    let k = next_usize();
    let q = next_usize();

    let mut cur_empty = Cell(0, 0);
    let mut board = Board::new(k, cur_empty);
    let graph = build_move_graph(&mut board, cur_empty);
    let lim = board.limit();

    if cfg!(LOCAL) {
        eprintln!("Board:");
        for i in 0..lim.0 {
            for j in 0..lim.1 {
                eprint!("{:2} ", board[Cell(i, j)]);
            }
            eprintln!();
        }
    }

    for _qid in 0..q {
        let r = next_usize();
        let c = next_usize();
        let target = Cell(r, c);

        let mut qu = VecDeque::<Cell>::new();
        qu.push_back(cur_empty);
        let mut dist = vec![None; lim.dim()];
        dist[cur_empty.encode(lim)] = Some(0);

        while let Some(cur) = qu.pop_front() {
            if cur == target {
                break;
            }
            let cur_dist = dist[cur.encode(lim)].unwrap();
            for &nei in graph[cur.encode(lim)].iter() {
                if dist[nei.encode(lim)].is_none() {
                    dist[nei.encode(lim)] = Some(cur_dist + 1);
                    qu.push_back(nei);
                }
            }
        }

        writeln!(writer, "{}", dist[Cell(r, c).encode(lim)].unwrap()).unwrap();
        cur_empty = target;
    }
}
