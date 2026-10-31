// Simulated-annealing level search: places a start pocket, a goal pocket and N rocks on an
// ice field so that the shortest solution is as long (and as unique) as possible.
//
//   rustc -O tools/search_level.rs -o search_level.exe
//   search_level W H ROCKS ITERS THREADS SEED TARGET_MOVES OUT.txt
//   python tools/levelkit.py build OUT.txt scenes/level_N.tscn
//
// The move rules mirror scripts/player.gd (see tools/levelkit.py, which is the reference).
use std::{env, fs, thread};

const PW: usize = 7;
const PH: usize = 6;
// Ladder pocket, copied from the start room in level_3: ladder on rows 1-2, open at the bottom.
const POCKET: [&str; PH] = ["#######", "#~.E.~#", "#~...~#", "#~.S.~#", "#~...~#", "#~~~~~#"];

const ICE: u8 = 0;
const GROUND: u8 = 1;
const WALL: u8 = 2;
const GOAL: u8 = 3; // ladder of the pocket we want to reach
const BACK: u8 = 4; // ladder of the start pocket (leaves the level: a dead end)

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

#[derive(Clone)]
struct State {
    pockets: [(usize, usize); 2], // [start, goal], top-left cell
    rocks: Vec<(usize, usize)>,
}

fn in_rect(p: (usize, usize), x: usize, y: usize) -> bool {
    x >= p.0 && x < p.0 + PW && y >= p.1 && y < p.1 + PH
}

fn overlap(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.0 + PW + 1 && b.0 < a.0 + PW + 1 && a.1 < b.1 + PH + 1 && b.1 < a.1 + PH + 1
}

fn free(st: &State, x: usize, y: usize) -> bool {
    !in_rect(st.pockets[0], x, y) && !in_rect(st.pockets[1], x, y) && !st.rocks.contains(&(x, y))
}

fn random_free(st: &State, rng: &mut Rng, w: usize, h: usize) -> (usize, usize) {
    loop {
        let (x, y) = (rng.below(w), rng.below(h));
        if free(st, x, y) {
            return (x, y);
        }
    }
}

fn build(st: &State, w: usize, h: usize, g: &mut [u8]) -> usize {
    let wp = w + 2;
    for (i, c) in g.iter_mut().enumerate() {
        let (x, y) = (i % wp, i / wp);
        *c = if x == 0 || y == 0 || x == wp - 1 || y == h + 1 { WALL } else { ICE };
    }
    for (i, p) in st.pockets.iter().enumerate() {
        for (r, row) in POCKET.iter().enumerate() {
            for (c, ch) in row.bytes().enumerate() {
                let k = match ch {
                    b'#' => WALL,
                    b'~' => ICE,
                    _ => GROUND,
                };
                g[(p.1 + r + 1) * wp + p.0 + c + 1] = k;
            }
        }
        for r in 1..=2 {
            g[(p.1 + r + 1) * wp + p.0 + 3 + 1] = if i == 1 { GOAL } else { BACK };
        }
    }
    for &(x, y) in &st.rocks {
        g[(y + 1) * wp + x + 1] = WALL;
    }
    (st.pockets[0].1 + 3 + 1) * wp + st.pockets[0].0 + 3 + 1
}

struct Eval {
    depth: i32,
    paths: u64,
    reached: usize,
}

fn evaluate(g: &[u8], wp: usize, start: usize, dist: &mut [i32], cnt: &mut [u64], queue: &mut Vec<usize>) -> Option<Eval> {
    dist.iter_mut().for_each(|d| *d = -1);
    queue.clear();
    queue.push(start);
    dist[start] = 0;
    cnt[start] = 1;
    let dirs = [-(wp as isize), wp as isize, -1, 1];
    let mut head = 0;
    while head < queue.len() {
        let p = queue[head];
        head += 1;
        if g[p] >= GOAL {
            continue;
        }
        for &d in &dirs {
            let t = (p as isize + d) as usize;
            let k = g[t];
            if k == WALL {
                continue;
            }
            let mut n = t;
            if k == ICE {
                while g[n] == ICE && g[(n as isize + d) as usize] != WALL {
                    n = (n as isize + d) as usize;
                }
            }
            if dist[n] < 0 {
                dist[n] = dist[p] + 1;
                cnt[n] = cnt[p];
                queue.push(n);
            } else if dist[n] == dist[p] + 1 {
                cnt[n] = cnt[n].saturating_add(cnt[p]);
            }
        }
    }
    let mut depth = i32::MAX;
    let mut paths = 0;
    for &c in queue.iter() {
        if g[c] == GOAL && dist[c] < depth {
            depth = dist[c];
            paths = 0;
        }
        if g[c] == GOAL && dist[c] == depth {
            paths += cnt[c];
        }
    }
    if depth == i32::MAX {
        None
    } else {
        Some(Eval { depth, paths, reached: queue.len() })
    }
}

fn score(e: &Eval, target: i32) -> f64 {
    -((e.depth - target).abs() as f64) - 0.7 * (e.paths.min(6) as f64 - 1.0) + 0.003 * e.reached as f64
}

fn ascii(st: &State, w: usize, h: usize) -> String {
    let mut rows = vec![vec![b'~'; w]; h];
    for &(x, y) in &st.rocks {
        rows[y][x] = b'#';
    }
    for (i, p) in st.pockets.iter().enumerate() {
        for (r, row) in POCKET.iter().enumerate() {
            for (c, ch) in row.bytes().enumerate() {
                rows[p.1 + r][p.0 + c] = if ch == b'S' && i == 1 { b'.' } else { ch };
            }
        }
    }
    rows.iter().map(|r| String::from_utf8(r.clone()).unwrap() + "\n").collect()
}

struct Best {
    score: f64,
    st: State,
    depth: i32,
    paths: u64,
}

fn search(w: usize, h: usize, rocks: usize, iters: usize, seed: u64, target: i32) -> Best {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let wp = w + 2;
    let n = wp * (h + 2);
    let (mut g, mut dist, mut cnt, mut queue) = (vec![0u8; n], vec![0i32; n], vec![0u64; n], Vec::new());

    let mut cur;
    let cur_eval;
    loop {
        let mut pk = [(0, 0); 2];
        loop {
            pk[0] = (rng.below(w - PW + 1), rng.below(h - PH + 1));
            pk[1] = (rng.below(w - PW + 1), rng.below(h - PH + 1));
            if !overlap(pk[0], pk[1]) {
                break;
            }
        }
        let mut st = State { pockets: pk, rocks: Vec::new() };
        while st.rocks.len() < rocks {
            let c = random_free(&st, &mut rng, w, h);
            st.rocks.push(c);
        }
        let start = build(&st, w, h, &mut g);
        if let Some(e) = evaluate(&g, wp, start, &mut dist, &mut cnt, &mut queue) {
            cur = st;
            cur_eval = e;
            break;
        }
    }
    let mut cur_score = score(&cur_eval, target);
    let mut best = Best { score: cur_score, st: cur.clone(), depth: cur_eval.depth, paths: cur_eval.paths };

    for it in 0..iters {
        let t = 3.0 * (0.03f64 / 3.0).powf(it as f64 / iters as f64);
        let mut cand = cur.clone();
        if rng.below(100) < 4 {
            let i = rng.below(2);
            let np = (rng.below(w - PW + 1), rng.below(h - PH + 1));
            if overlap(np, cand.pockets[1 - i]) {
                continue;
            }
            cand.pockets[i] = np;
            for r in 0..cand.rocks.len() {
                if in_rect(np, cand.rocks[r].0, cand.rocks[r].1) {
                    cand.rocks[r] = (usize::MAX, usize::MAX);
                    let c = random_free(&cand, &mut rng, w, h);
                    cand.rocks[r] = c;
                }
            }
        } else {
            let r = rng.below(cand.rocks.len());
            let old = cand.rocks[r];
            cand.rocks[r] = (usize::MAX, usize::MAX);
            cand.rocks[r] = if rng.below(2) == 0 {
                // nudge a rock one or two cells
                let dx = rng.below(5) as isize - 2;
                let dy = rng.below(5) as isize - 2;
                let (x, y) = (old.0 as isize + dx, old.1 as isize + dy);
                if x < 0 || y < 0 || x >= w as isize || y >= h as isize || !free(&cand, x as usize, y as usize) {
                    old
                } else {
                    (x as usize, y as usize)
                }
            } else {
                random_free(&cand, &mut rng, w, h)
            };
        }
        let start = build(&cand, w, h, &mut g);
        let Some(e) = evaluate(&g, wp, start, &mut dist, &mut cnt, &mut queue) else { continue };
        let s = score(&e, target);
        if s >= cur_score || rng.unit() < ((s - cur_score) / t).exp() {
            cur = cand;
            cur_score = s;
            if s > best.score {
                best = Best { score: s, st: cur.clone(), depth: e.depth, paths: e.paths };
            }
        }
    }
    best
}

fn main() {
    let a: Vec<String> = env::args().collect();
    if a.len() != 9 {
        eprintln!("usage: search_level W H ROCKS ITERS THREADS SEED TARGET_MOVES OUT.txt");
        return;
    }
    let p = |i: usize| a[i].parse::<usize>().unwrap();
    let (w, h, rocks, iters, threads, seed, target) = (p(1), p(2), p(3), p(4), p(5), p(6) as u64, p(7) as i32);
    let handles: Vec<_> = (0..threads)
        .map(|i| thread::spawn(move || search(w, h, rocks, iters, seed * 1000 + i as u64, target)))
        .collect();
    let mut best: Option<Best> = None;
    for hd in handles {
        let b = hd.join().unwrap();
        eprintln!("thread: depth {} shortest-paths {} score {:.2}", b.depth, b.paths, b.score);
        if best.as_ref().map_or(true, |x| b.score > x.score) {
            best = Some(b);
        }
    }
    let b = best.unwrap();
    let text = format!(
        "; {}x{} field, {} rocks, shortest solution {} moves ({} optimal route(s))\n{}",
        w, h, rocks, b.depth, b.paths, ascii(&b.st, w, h)
    );
    fs::write(&a[8], text).unwrap();
}
