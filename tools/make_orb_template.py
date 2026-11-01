"""Writes the orb template for tools/search_template.rs.

# wall  ~ ice a rock may sit on  i fixed ice  . ground  = band cell (wall unless the search opens its column)
E start ladder  G goal ladder  S spawn
"""
import sys

N, R = 40, 19.0
C = (N - 1) / 2
START_TOP, GOAL_TOP = 3, 17
POCKET = ["#######", "#i.E.i#", "#i...i#", "#i.S.i#", "#i...i#", "#iiiii#"]
GOAL = ["#######", "#i.G.i#", "#i...i#", "#i...i#", "#i...i#", "#iiiii#"]

rows = [["#"] * N for _ in range(N)]
for y in range(N):
    for x in range(N):
        if (x - C) ** 2 + (y - C) ** 2 <= R * R:
            rows[y][x] = "=" if y in (19, 20) else "~"
for top, pocket in ((START_TOP, POCKET), (GOAL_TOP, GOAL)):
    for dy, line in enumerate(pocket):
        for dx, ch in enumerate(line):
            rows[top + dy][16 + dx] = ch
open(sys.argv[1], "w").write("\n".join("".join(r) for r in rows) + "\n")
