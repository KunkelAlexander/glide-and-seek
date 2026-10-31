# Glide and Seek - notes for Claude

Godot 4 grid puzzle: step onto ice, slide until something stops you, reach the ladder.
`index.*` files in the repo root are the committed web export; do not edit them by hand.

## Game rules (scripts/player.gd, scripts/grid.gd)
- Tile types come from the Logic layer's custom data `type`: `ice`, `ground`, `wall`, `exit` (void = no tile, treated as walkable, so always enclose levels in walls).
- Moving onto ground/exit walks one tile. Moving onto ice slides: the player keeps going while the current tile is ice and the next tile is not a wall, and stops on the first non-ice tile it enters.
- Exits fire only when the player *stops* on one (an exit tile is non-ice, so every slide stops there). Ladders are 2 tiles tall; both tiles are exits.
- Every level has a start ladder (`SpawnPoint` in group `entrance_a`, next to it) and a goal ladder. Both load `level_1.tscn`, the hub. The puzzle is reaching the far ladder.
- `level_1.tscn` is the hub: one ladder per level, selected by logic tile (see below).

## Level scene layout
`Background` (rock fill) / `Background2` (ice floor and ladder-room frames) / `Foreground` (rim, rocks, ladders) are visuals only. **`Logic`** (hidden) is the only layer the game reads. `Grid` node points at it. Keep visuals and logic in step.

Logic tileset = `assets/logic_tileset_32x32.png`, source 2 in `assets/ice.tres`. Atlas coords: (0,0) ground, (1,0) ice, (2,0) wall, (3,0) exit to level_1; hub selector exits (0,1)=level_2, (1,1)=level_3, (2,1)=level_4, (3,1)=level_5, (4,1)=level_6. A new hub ladder needs a new tile entry with `type="exit"`, `target`, `spawn="entrance_a"` in `ice.tres`. Edit it inside the *second* atlas source only: the first (visual) source reuses the same `N:M/0` keys.

Visual tiles (source 0, `assets/ice.png`): rock (5,19); ladder (6,7)+(6,8); hub ladder frame (5..7, 13..14); ice 9-patch edges (0..2, 13..15); ladder-room snow (1,4).

## Level tooling (tools/, ignored by Godot via .gdignore)
Hand-drawing in the TileMap editor is still fine. For hard puzzles use the pipeline:

1. **Search** (optional): `rustc -O tools/search_level.rs -o search_level.exe`, then `search_level W H ROCKS ITERS THREADS SEED TARGET_MOVES out.txt`. Annealing over rock positions and the two ladder pockets; aims for a shortest solution near TARGET_MOVES with a single optimal route. 30 22 45 600000 8 seed 52 takes seconds. Compile output outside the repo.
2. **ASCII** in `tools/levels/level_N.txt`: `~` ice, `.` ground, `#` wall/rock, `E` ladder (upper tile; the lower is implied), `S` spawn. `;` lines are comments. Keep the solution in a comment.
3. **Build**: `python tools/levelkit.py build tools/levels/level_N.txt scenes/level_N.tscn` writes all four layers and the spawn from the `level_5.tscn` template. Re-running keeps the scene's uid.
4. **Solve/verify**: `python tools/levelkit.py solve <scene.tscn or ascii>` prints the shortest key presses to every ladder. It mirrors player.gd. Use it on hand-edited scenes too.
5. Wire into the hub (logic tile + `Foreground` ladder frame above the Logic exit tiles, as done for level_6 at x=15, y=4..5) and add the target tile to `ice.tres`.

Difficulty reference (shortest solution, moves): level_2 53, level_3 22, level_4 30, level_5 24, level_6 52. Levels 2-5 were hardened by adding a few rocks (3, 6, 5, 3) to the original layouts.

## Gotchas
- `tile_map_data` is base64: 2-byte header then 12 bytes per cell (`x,y,source,atlas_x,atlas_y,alt`, int16 LE). Godot reads the **last** `tile_map_data` in a node, so replace in place; never add a second one. The Logic node lists `visible`/`modulate` before its data.
- Each level's field uses origin (-1, 1) so the camera framing matches level_3; Background fill is x -42..76, y -18..46.
- Verification must run on the real engine data, not just the decoder: a bug once made the scene wrong while the Python reader (which shares the codec) still looked right.
- Do not leave `.import` rewrites in git. Godot 4.7 resaves them when it imports; the project targets 4.4. After any `--import` run `git checkout -- '*.import'`.
- A headless run needs `.godot/` imported first: `Godot_..._console.exe --headless --path . --import`.

## Godot on this machine
Executable: `C:\Users\TE\Downloads\Godot_v4.7.2-stable_win64.exe\Godot_v4.7.2-stable_win64.exe` (a folder named like the exe; the console variant is beside it). `.mcp.json` configures `@coding-solo/godot-mcp` with that path (tools: run/launch/create scene/add node/uids; no GDScript editing).
- Replay a solution on real data: a `SceneTree` script run with `--headless -s` that loads the level, uses `Grid.is_ice/is_blocked/is_exit` and the player.gd step logic.
- Screenshot a level: run a `-s` script **without** `--headless` that adds a `Camera2D`, awaits `RenderingServer.frame_post_draw`, and saves `root.get_texture().get_image()` to a PNG.
