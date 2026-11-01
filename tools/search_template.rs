// Annealing search on a fixed template (see tools/make_orb_template.py for the legend).
// Places ROCKS rocks on the '~' cells and opens GAPS columns of the '=' band so that the shortest
// route from S to the goal ladder G is long and has as few optimal routes as possible.
//
//   rustc -O tools/search_template.rs -o search_template.exe
//   search_template TEMPLATE.txt ROCKS GAPS ITERS THREADS SEED TARGET_MOVES OUT.txt
//
// The move rules mirror scripts/player.gd (tools/levelkit.py is the reference).
use std::{collections::VecDeque, env, fs, thread};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        ((self.next() >> 33) as usize) % n
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

const ICE: u8 = 0;
const GROUND: u8 = 1;
const WALL: u8 = 2;
const EXIT: u8 = 3;

#[derive(Clone)]
struct Model {
    w: usize,
    h: usize,
    base: Vec<u8>,          // padded grid (1-cell wall border)
    spawn: usize,
    goal: [usize; 2],       // both tiles of the goal ladder
    rock_cells: Vec<usize>, // candidate rock positions
    band_cols: Vec<[usize; 2]>,
}

fn parse(text: &str) -> Model {
    let rows: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    let (h, w) = (rows.len() + 2, rows[0].len() + 2);
    let mut m = Model { w, h, base: vec![WALL; w * h], spawn: 0, goal: [0, 0], rock_cells: vec![], band_cols: vec![] };
    let mut band: Vec<(usize, usize)> = vec![];
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let i = (y + 1) * w + x + 1;
            m.base[i] = match ch {
                '#' | '=' => WALL,
                '~' | 'i' => ICE,
                '.' => GROUND,
                'S' => GROUND,
                'E' => EXIT,
                'G' => EXIT,
                _ => panic!("bad char {ch}"),
            };
            match ch {
                'S' => m.spawn = i,
                'G' => m.goal = [i, i + w],
                '~' => m.rock_cells.push(i),
                '=' => band.push((x, i)),
                _ => {}
            }
        }
    }
    for &(x, i) in &band {
        if let Some(&(_, j)) = band.iter().find(|&&(x2, j)| x2 == x && j == i + w) {
            m.band_cols.push([i, j]);
        }
    }
    let exits: Vec<usize> = (0..w * h).filter(|&i| m.base[i] == EXIT).collect();
    for e in exits {
        if e + w < w * h {
            m.base[e + w] = EXIT; // ladders are 2 tiles tall
        }
    }
    m
}

fn step(g: &[u8], w: usize, pos: usize, d: isize) -> usize {
    let t = (pos as isize + d) as usize;
    if g[t] == WALL {
        return pos;
    }
    if g[t] != ICE {
        return t;
    }
    let mut cur = t;
    while g[cur] == ICE && g[(cur as isize + d) as usize] != WALL {
        cur = (cur as isize + d) as usize;
    }
    let _ = w;
    cur
}

struct Eval {
    dist: i32,
    paths: u64,
    reach: usize,
    route: Vec<usize>,
}

fn eval(m: &Model, g: &[u8]) -> Option<Eval> {
    let n = g.len();
    let dirs = [-(m.w as isize), m.w as isize, -1, 1];
    let mut dist = vec![-1i32; n];
    let mut cnt = vec![0u64; n];
    let mut prev = vec![usize::MAX; n];
    let mut q = VecDeque::new();
    dist[m.spawn] = 0;
    cnt[m.spawn] = 1;
    q.push_back(m.spawn);
    while let Some(p) = q.pop_front() {
        if g[p] == EXIT {
            continue;
        }
        for &d in &dirs {
            let t = step(g, m.w, p, d);
            if t == p {
                continue;
            }
            if dist[t] < 0 {
                dist[t] = dist[p] + 1;
                prev[t] = p;
                q.push_back(t);
            }
            if dist[t] == dist[p] + 1 {
                cnt[t] = cnt[t].saturating_add(cnt[p]);
            }
        }
    }
    let best = m.goal.iter().filter(|&&c| dist[c] >= 0).min_by_key(|&&c| dist[c])?;
    let mut route = vec![*best];
    let mut c = *best;
    while prev[c] != usize::MAX {
        c = prev[c];
        route.push(c);
    }
    route.reverse();
    let paths: u64 = m.goal.iter().filter(|&&c| dist[c] == dist[*best]).map(|&c| cnt[c]).sum();
    Some(Eval { dist: dist[*best], paths, reach: dist.iter().filter(|&&d| d >= 0).count(), route })
}

fn build(m: &Model, rocks: &[usize], open: &[usize]) -> Vec<u8> {
    let mut g = m.base.clone();
    for &r in rocks {
        g[r] = WALL;
    }
    for &c in open {
        g[m.band_cols[c][0]] = ICE;
        g[m.band_cols[c][1]] = ICE;
    }
    g
}

fn score(e: &Option<Eval>, target: i32) -> f64 {
    match e {
        None => -1e9,
        Some(e) => e.dist.min(target) as f64 - 2.0 * (e.paths.min(11) - 1) as f64 + 0.01 * e.reach as f64,
    }
}

fn anneal(m: &Model, rocks_n: usize, gaps_n: usize, iters: usize, seed: u64, target: i32) -> (f64, Vec<usize>, Vec<usize>) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut rocks: Vec<usize> = vec![];
    while rocks.len() < rocks_n {
        let c = m.rock_cells[rng.below(m.rock_cells.len())];
        if !rocks.contains(&c) {
            rocks.push(c);
        }
    }
    let mut open: Vec<usize> = vec![];
    while open.len() < gaps_n {
        let c = rng.below(m.band_cols.len());
        if !open.contains(&c) {
            open.push(c);
        }
    }
    let mut cur = score(&eval(m, &build(m, &rocks, &open)), target);
    let (mut best, mut br, mut bo) = (cur, rocks.clone(), open.clone());
    for it in 0..iters {
        let temp = 3.0 * (0.03f64 / 3.0).powf(it as f64 / iters as f64);
        let (mut nr, mut no) = (rocks.clone(), open.clone());
        match rng.below(10) {
            0 => {
                let c = rng.below(m.band_cols.len());
                if !no.contains(&c) {
                    let k = rng.below(no.len());
                    no[k] = c;
                }
            }
            _ => {
                let c = m.rock_cells[rng.below(m.rock_cells.len())];
                if !nr.contains(&c) {
                    let k = rng.below(nr.len());
                    nr[k] = c;
                }
            }
        }
        let s = score(&eval(m, &build(m, &nr, &no)), target);
        if s >= cur || rng.unit() < ((s - cur) / temp).exp() {
            rocks = nr;
            open = no;
            cur = s;
            if cur > best {
                best = cur;
                br = rocks.clone();
                bo = open.clone();
            }
        }
    }
    (best, br, bo)
}

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 9 {
        eprintln!("usage: search_template TEMPLATE ROCKS GAPS ITERS THREADS SEED TARGET OUT");
        return;
    }
    let text = fs::read_to_string(&a[1]).unwrap();
    let m = parse(&text);
    let (rn, gn, iters, threads, seed, target): (usize, usize, usize, u64, u64, i32) =
        (a[2].parse().unwrap(), a[3].parse().unwrap(), a[4].parse().unwrap(), a[5].parse().unwrap(), a[6].parse().unwrap(), a[7].parse().unwrap());
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let m = m.clone();
            thread::spawn(move || anneal(&m, rn, gn, iters, seed * 1000 + t, target))
        })
        .collect();
    let (mut best, mut br, mut bo) = (-1e18, vec![], vec![]);
    for h in handles {
        let (s, r, o) = h.join().unwrap();
        println!("chain score {s:.2}");
        if s > best {
            (best, br, bo) = (s, r, o);
        }
    }
    let g = build(&m, &br, &bo);
    let e = eval(&m, &g).expect("unsolvable");
    println!("shortest {} moves, {} optimal route(s), {} reachable tiles", e.dist, e.paths, e.reach);
    let src = text.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let mut out = String::new();
    for (y, row) in src.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let i = (y + 1) * m.w + x + 1;
            out.push(match ch {
                '~' if g[i] == WALL => '#',
                '~' | 'i' => '~',
                '=' => if g[i] == WALL { '#' } else { '~' },
                c => c,
            });
        }
        out.push('\n');
    }
    let route: String = e
        .route
        .windows(2)
        .map(|p| {
            let (a, b) = (p[0] as isize, p[1] as isize);
            let (dx, dy) = ((b % m.w as isize) - (a % m.w as isize), (b / m.w as isize) - (a / m.w as isize));
            if dx.abs() >= dy.abs() { if dx > 0 { 'R' } else { 'L' } } else if dy > 0 { 'D' } else { 'U' }
        })
        .collect();
    fs::write(&a[8], out).unwrap();
    println!("route {route}");
}
