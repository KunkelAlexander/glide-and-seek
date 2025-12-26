extends Node2D

@export var tile_size: int = 32
@export var world: TileMapLayer

func _ready():
	assert(world != null)

	
# ─────────────────────────────
# Coordinate conversion
# ─────────────────────────────

func grid_to_world(cell: Vector2i) -> Vector2:
	return Vector2(
		cell.x * tile_size + tile_size / 2,
		cell.y * tile_size + tile_size / 2
	)

func world_to_grid(pos: Vector2) -> Vector2i:
	return Vector2i(
		floor(pos.x / tile_size),
		floor(pos.y / tile_size)
	)

# ─────────────────────────────
# Tile queries
# ─────────────────────────────

func get_tile_type(cell: Vector2i) -> String:
	var tile_data := world.get_cell_tile_data(cell)
	if tile_data == null:
		return "void"
	return tile_data.get_custom_data("type")

func is_wall(cell: Vector2i) -> bool:
	return get_tile_type(cell) == "wall"

func is_ice(cell: Vector2i) -> bool:
	return get_tile_type(cell) == "ice"

func is_goal(cell: Vector2i) -> bool:
	return get_tile_type(cell) == "goal"

func is_blocked(cell: Vector2i) -> bool:
	print("Is blocked? Cell: ", cell)
	return is_wall(cell)

func is_exit(cell: Vector2i) -> bool:
	return get_tile_type(cell) == "exit"

func get_exit_data(cell: Vector2i) -> Dictionary:
	var data := world.get_cell_tile_data(cell)
	if data == null:
		return {}
	return {
		"target": data.get_custom_data("target"),
		"spawn": data.get_custom_data("spawn")
	}
