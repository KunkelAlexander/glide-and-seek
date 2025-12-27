# Glide and seek

A tiny Pokémon–style ice cave prototype in Godot: step onto ice, commit to the slide, and try not to get stuck.
I used to love these in the Pokémon games.

## Features
- Built two small levels using the TileMap editor.
- Implemented grid-locked movement + ice sliding (no physics bodies).
- Added ladders as exits to swap levels and respawn at entrances.
- Added a camera that follows the player.
- Split level editing into a functional “logic layer” and decorative layers.
- Hooked up background music.

## Devlog

### 1. How to make levels?
<img src="figures/Screenshot from 2025-12-27 08-43-14.png" width="800" alt="Simple level building using the spriteset">

Simple level building using the spriteset.

### 2. How to add a player?
<img src="figures/Screenshot from 2025-12-27 08-44-19.png" width="800" alt="The player sprite">

The player sprite.

### 3. How to change between levels?

<img src="figures/Screenshot from 2025-12-27 08-45-01.png" width="800" alt="The exit via a ladder">

Add the exit via a ladder. Currently, the tileset encodes which level loads and where the player spawns.


### 4. How to handle different screen sizes?

<img src="figures/Screenshot from 2025-12-27 08-49-09.png" width="800" alt="A dynamic camera following the player">

Add a dynamic camera following the player. This means that the game can run with different screen sizes.

### 5. How to improve level editing?

<img src="figures/Screenshot from 2025-12-27 10-31-54.png" width="800" alt="A separate logical tilemap layer describing the level logic">

Add separate logical tilemap layer describing the level logic. This means that the visuals and the level logic are decoupled.

## How to run

### Godot
1. Open the project in **Godot 4.4**.
2. Run `scenes/main.tscn`.

### Web export
A web build is included (`index.html`, `index.wasm`, `index.pck`). Serve the folder with a local web server and open `index.html`.


## Controls
- Arrow keys: move
- R key: Reset player position
- (On ice) movement continues until you leave the ice or hit a wall


## Project layout

```

trapped-in-ice
├─ assets/                  # sprites, tilesets, music
├─ figures/                 # screenshots for this README
├─ scenes/
│  ├─ main.tscn             # persistent root (player, level manager, camera, fade/audio)
│  ├─ level_1.tscn
│  └─ level_2.tscn
└─ scripts/
├─ grid.gd               # tile queries + grid/world conversion
├─ player.gd             # movement + sliding + animation hooks
└─ level_manager.gd      # level loading + spawn + transitions

```


## Notes (design)
- Levels are authored in the TileMap editor.
- Logic is kept on a plain-color TileMap layer (walls / ice / exits), with visuals layered on top.
- Level loading is handled by `LevelManager`, which injects the new `Grid` reference into the persistent player.

## Acknowledgements
- Cave tiles: https://www.deviantart.com/descendedfromullr/art/Ice-Cave-Tileset-823235944
    - Cave Textures - Phyromatical - https://www.deviantart.com/phyromatical
    - Cave Textures - EVoLiNa - https://www.deviantart.com/evolina
- Player sprite: https://jordiebowen.weebly.com/3d-modelling--animation/3d-pixel-character
- Background music: https://opengameart.org/content/ice-shine-bells