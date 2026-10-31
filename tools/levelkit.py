"""Level tooling for Glide and Seek: ASCII -> scene builder and an exact solver.

    python tools/levelkit.py solve scenes/level_3.tscn        # optimal solution per ladder
    python tools/levelkit.py solve tools/levels/level_6.txt   # works on ASCII sources too
    python tools/levelkit.py build tools/levels/level_6.txt scenes/level_6.tscn

ASCII legend (one character per 32x32 tile, ';' starts a comment line):
    ~ ice     . ground     # wall / rock     E ladder (2 tiles tall, upper tile marked)     S spawn (on ground)

The solver mirrors scripts/player.gd exactly: a move onto ice slides until the
next tile is a wall or the player leaves the ice; ladders end the level.
"""
import base64
import random
import re
import struct
import sys
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TILE = 32

# Logic tileset (source 2 in assets/ice.tres)
LOGIC_SRC = 2
GROUND, ICE, WALL, EXIT = (0, 0), (1, 0), (2, 0), (3, 0)
EXIT_TILES = {(3, 0), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1)}

# Field origin in world cells, matching level_3.tscn so the camera framing is the same
X0, Y0 = -1, 1
BG_EXTENT = (-42, 76, -18, 46)  # Background fill, same as every other level
PAD = 3  # wall ring around the field in the Logic layer

DIRS = {"U": (0, -1), "D": (0, 1), "L": (-1, 0), "R": (1, 0)}


# ───────────── tile_map_data codec ─────────────

def decode(b64):
    raw = base64.b64decode(b64)
    cells = {}
    for i in range(2, len(raw), 12):
        x, y, src, ax, ay, _alt = struct.unpack("<hhhhhh", raw[i:i + 12])
        cells[(x, y)] = (src, ax, ay)
    return cells


def encode(cells):
    out = bytearray(2)
    for (x, y), (src, ax, ay) in cells.items():
        out += struct.pack("<hhhhhh", x, y, src, ax, ay, 0)
    return base64.b64encode(bytes(out)).decode()


def scene_layers(text):
    layers = {}
    for m in re.finditer(r'\[node name="(\w+)" type="TileMapLayer".*?(?=\n\[node|\Z)', text, re.S):
        d = re.search(r'tile_map_data = PackedByteArray\("([^"]*)"\)', m.group(0))
        layers[m.group(1)] = decode(d.group(1)) if d else {}
    return layers


# ───────────── grids (shared by solver and builder) ─────────────
# A grid is {(x, y): 'ice' | 'ground' | 'wall' | 'exit'}; missing cells are void.

def grid_from_scene(path):
    text = Path(path).read_text(encoding="utf8")
    grid = {}
    for cell, (src, ax, ay) in scene_layers(text)["Logic"].items():
        if src != LOGIC_SRC:
            continue
        if (ax, ay) == ICE:
            grid[cell] = "ice"
        elif (ax, ay) == WALL:
            grid[cell] = "wall"
        elif (ax, ay) in EXIT_TILES:
            grid[cell] = "exit"
        else:
            grid[cell] = "ground"
    m = re.search(r"SpawnPoint.*?position = Vector2\(([\d.\-]+), ([\d.\-]+)\)", text, re.S)
    spawn = (int(float(m.group(1))) // TILE, int(float(m.group(2))) // TILE)
    return grid, spawn


def read_ascii(path):
    rows = [ln.rstrip("\n") for ln in Path(path).read_text(encoding="utf8").splitlines()
            if ln.strip() and not ln.startswith(";")]
    if len({len(r) for r in rows}) != 1:
        raise SystemExit("ASCII rows must all have the same width")
    return rows


def grid_from_ascii(rows):
    names = {"~": "ice", ".": "ground", "#": "wall", "E": "exit", "S": "ground"}
    grid, spawn = {}, None
    for ry, row in enumerate(rows):
        for rx, ch in enumerate(row):
            cell = (rx + X0, ry + Y0)
            grid[cell] = names[ch]
            if ch == "S":
                spawn = cell
    # every ladder is 2 tiles tall; the second tile sits below the marked one
    for cell in [c for c, k in grid.items() if k == "exit"]:
        grid[(cell[0], cell[1] + 1)] = "exit"
    # the builder surrounds the field with walls; mirror that so slides stop at the edge
    h, w = len(rows), len(rows[0])
    for x in range(X0 - 1, X0 + w + 1):
        grid[(x, Y0 - 1)] = grid[(x, Y0 + h)] = "wall"
    for y in range(Y0 - 1, Y0 + h + 1):
        grid[(X0 - 1, y)] = grid[(X0 + w, y)] = "wall"
    return grid, spawn


# ───────────── solver ─────────────

def step(grid, pos, d):
    dx, dy = DIRS[d]
    t = (pos[0] + dx, pos[1] + dy)
    k = grid.get(t, "void")
    if k == "wall":
        return pos
    if k != "ice":
        return t
    cur = t
    while grid.get(cur) == "ice" and grid.get((cur[0] + dx, cur[1] + dy), "void") != "wall":
        cur = (cur[0] + dx, cur[1] + dy)
    return cur


def bfs(grid, spawn):
    """Shortest key-press count to every reachable resting tile. Ladders are terminal."""
    prev, dist, q = {spawn: None}, {spawn: 0}, deque([spawn])
    while q:
        p = q.popleft()
        if grid.get(p) == "exit":
            continue
        for d in "UDLR":
            n = step(grid, p, d)
            if n != p and n not in dist:
                dist[n], prev[n] = dist[p] + 1, (p, d)
                q.append(n)
    return prev, dist


def path_to(prev, cell):
    out = []
    while prev[cell]:
        cell, d = prev[cell]
        out.append(d)
    return "".join(reversed(out))


def cmd_solve(src):
    if src.endswith(".tscn"):
        grid, spawn = grid_from_scene(src)
    else:
        grid, spawn = grid_from_ascii(read_ascii(src))
    prev, dist = bfs(grid, spawn)
    exits = sorted((dist[c], c) for c in dist if grid[c] == "exit")
    print(f"{src}: spawn {spawn}, {len(dist)} reachable resting tiles")
    seen = set()
    for d, c in exits:
        # the two tiles of one ladder have adjacent cells; report each ladder once
        key = (c[0], c[1] // 2)
        if (c[0], c[1] - 1) in seen or (c[0], c[1] + 1) in seen:
            continue
        seen.add(c)
        print(f"  ladder at {c}: {d} moves  {path_to(prev, c)}")


# ───────────── scene builder ─────────────

def ladder_pocket_cells(ex, ey):
    """Background2 stamp for a ladder pocket, copied from the start room of level_3."""
    cells = {}
    ix0, iy0 = ex - 2, ey  # 5x5 interior
    for dy in range(5):
        for dx in range(5):
            if dy == 0:
                t = (3, 13) if dx == 0 else (4, 13) if dx == 4 else (1, 15)
            elif dy == 4:
                t = (3, 14) if dx == 0 else (4, 14) if dx == 4 else (1, 13)
            else:
                t = (2, 14) if dx == 0 else (0, 14) if dx == 4 else (1, 4)
            cells[(ix0 + dx, iy0 + dy)] = (0,) + t
    return cells


def build_layers(rows):
    grid, spawn = grid_from_ascii(rows)
    w, h = len(rows[0]), len(rows)
    x1, y1 = X0 + w - 1, Y0 + h - 1

    bg0, bg1, by0, by1 = BG_EXTENT
    background = {(x, y): (0, 1, 1) for x in range(bg0, bg1 + 1) for y in range(by0, by1 + 1)}

    floor = {}
    for x in range(X0, x1 + 1):
        for y in range(Y0, y1 + 1):
            cx = 0 if x == X0 else 2 if x == x1 else 1
            cy = 13 if y == Y0 else 15 if y == y1 else 14
            floor[(x, y)] = (0, cx, cy)

    fore = {}
    # cliff rim around the field (same tiles level_3 uses)
    fore[(X0 - 1, Y0 - 2)], fore[(x1 + 1, Y0 - 2)] = (0, 3, 3), (0, 4, 3)
    fore[(X0 - 1, y1 + 1)], fore[(x1 + 1, y1 + 1)] = (0, 3, 1), (0, 4, 1)
    for x in range(X0, x1 + 1):
        fore[(x, Y0 - 2)], fore[(x, Y0 - 1)], fore[(x, y1 + 1)] = (0, 1, 5), (0, 1, 6), (0, 1, 0)
    for y in range(Y0 - 1, y1 + 1):
        near_top = y <= Y0
        fore[(X0 - 1, y)] = (0, 3, 4) if near_top else (0, 2, 1) if y == y1 else (0, 2, 4)
        fore[(x1 + 1, y)] = (0, 4, 4) if near_top else (0, 0, 4)

    logic = {}
    for x in range(X0 - PAD - 1, x1 + PAD + 2):
        for y in range(Y0 - PAD - 1, y1 + PAD + 2):
            logic[(x, y)] = (LOGIC_SRC,) + WALL
    names = {"ice": ICE, "ground": GROUND, "wall": WALL}
    for cell, kind in grid.items():
        if kind == "exit":
            logic[cell] = (LOGIC_SRC,) + EXIT
        else:
            logic[cell] = (LOGIC_SRC,) + names[kind]
        if kind == "wall" and X0 <= cell[0] <= x1 and Y0 <= cell[1] <= y1:  # rocks only; the rim is drawn above
            fore[cell] = (0, 5, 19)

    for cell in [c for c, k in grid.items() if k == "exit" and grid.get((c[0], c[1] - 1)) != "exit"]:
        floor.update(ladder_pocket_cells(*cell))
        fore[cell], fore[(cell[0], cell[1] + 1)] = (0, 6, 7), (0, 6, 8)

    return {"Background": background, "Background2": floor, "Foreground": fore, "Logic": logic}, spawn


def new_uid(rng):
    return "uid://" + "".join(rng.choice("abcdefghijklmnopqrstuvwxy012345678") for _ in range(12))


def cmd_build(ascii_path, out_path, template=ROOT / "scenes" / "level_5.tscn"):
    rows = read_ascii(ascii_path)
    layers, spawn = build_layers(rows)
    text = template.read_text(encoding="utf8")
    for name, cells in layers.items():
        m = re.search(r'\[node name="%s" type="TileMapLayer".*?(?=\n\[|\Z)' % name, text, re.S)
        line = f'tile_map_data = PackedByteArray("{encode(cells)}")'
        block = m.group(0)
        if "tile_map_data" in block:  # replace in place: Godot reads the last tile_map_data it finds
            block = re.sub(r'tile_map_data = PackedByteArray\("[^"]*"\)', lambda _: line, block, count=1)
        else:
            head, rest = block.split("\n", 1)
            block = head + "\n" + line + "\n" + rest
        text = text[:m.start()] + block + text[m.end():]
    text = re.sub(r"(SpawnPoint.*?position = Vector2\()[\d.\-]+, [\d.\-]+", rf"\g<1>{spawn[0] * TILE + TILE // 2}, {spawn[1] * TILE + TILE // 2}", text, count=1, flags=re.S)
    out = Path(out_path)
    uid = re.search(r'uid="(uid://[^"]+)"', out.read_text(encoding="utf8")).group(1) if out.exists() else new_uid(random.Random(out.name))
    text = re.sub(r'(\[gd_scene [^\]]*uid=")uid://[^"]+', lambda m: m.group(1) + uid, text, count=1)
    out.write_text(text, encoding="utf8", newline="\n")
    print(f"wrote {out} ({len(rows[0])}x{len(rows)} field, spawn {spawn})")


if __name__ == "__main__":
    if len(sys.argv) >= 3 and sys.argv[1] == "solve":
        for s in sys.argv[2:]:
            cmd_solve(s)
    elif len(sys.argv) == 4 and sys.argv[1] == "build":
        cmd_build(sys.argv[2], sys.argv[3])
    else:
        print(__doc__)
