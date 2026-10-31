"""Render an SVG solution diagram per level: python tools/solution_diagrams.py [out_dir]

Reads each scene's Logic layer, solves it with levelkit's solver and draws the optimal
route. Numbered dots are the resting tiles after each key press; lines show the slides.
"""
import sys
from pathlib import Path

import levelkit as L

LEVELS = (2, 3, 4, 5, 6)
COLORS = {"ice": "#cfe8f5", "ground": "#d9c9a3", "wall": "#4b4f57", "exit": "#f0a030"}


def route(grid, spawn):
    prev, dist = L.bfs(grid, spawn)
    goals = sorted((d, c) for c, d in dist.items() if grid[c] == "exit" and c[0] != spawn[0])
    goal = goals[0][1]
    moves, stops = L.path_to(prev, goal), [spawn]
    for m in moves:
        stops.append(L.step(grid, stops[-1], m))
    return moves, stops


def render(n, out):
    grid, spawn = L.grid_from_scene(ROOT / "scenes" / f"level_{n}.tscn")
    moves, stops = route(grid, spawn)
    cells = [c for c, k in grid.items() if k != "wall"]
    x0, x1 = min(c[0] for c in cells) - 1, max(c[0] for c in cells) + 1
    y0, y1 = min(c[1] for c in cells) - 1, max(c[1] for c in cells) + 1
    s = max(12, min(20, 880 // (x1 - x0 + 1)))
    w, h = (x1 - x0 + 1) * s, (y1 - y0 + 1) * s
    mid = lambda c: ((c[0] - x0 + .5) * s, (c[1] - y0 + .5) * s)
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h + 22}" width="{w}" font-family="sans-serif">',
           f'<rect width="{w}" height="{h + 22}" fill="#fff"/>']
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            k = grid.get((x, y), "wall")
            svg.append(f'<rect x="{(x - x0) * s}" y="{(y - y0) * s}" width="{s}" height="{s}" fill="{COLORS[k]}" stroke="#fff" stroke-width=".4"/>')
    px, py = mid(spawn)
    svg.append(f'<circle cx="{px}" cy="{py}" r="{s * .45}" fill="#3a9d4a"/>')
    pts = " ".join("%.1f,%.1f" % mid(c) for c in stops)
    svg.append(f'<polyline points="{pts}" fill="none" stroke="#d62828" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" opacity=".85"/>')
    for i, c in enumerate(stops[1:], 1):
        cx, cy = mid(c)
        svg.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="{s * .36:.1f}" fill="#d62828"/>'
                   f'<text x="{cx:.1f}" y="{cy + s * .13:.1f}" font-size="{s * .38:.1f}" fill="#fff" text-anchor="middle" font-weight="bold">{i}</text>')
    svg.append(f'<text x="4" y="{h + 15}" font-size="13" fill="#222">Level {n}: {len(moves)} moves  {moves}</text></svg>')
    Path(out).write_text("\n".join(svg), encoding="utf8")
    print(f"wrote {out} ({len(moves)} moves)")


ROOT = L.ROOT
if __name__ == "__main__":
    d = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "tools" / "solutions"
    d.mkdir(parents=True, exist_ok=True)
    for n in LEVELS:
        render(n, d / f"level_{n}.svg")
