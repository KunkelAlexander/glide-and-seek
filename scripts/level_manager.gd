extends Node

@export var fade_time := 0.4

var transitioning := false

@export var starting_level := "res://scenes/level_1.tscn"
@export var starting_spawn := "entrance_a"

var current_level: Node = null


func _ready():
	
	# Defer loading until the scene tree is fully ready
	call_deferred("_load_initial_level")

func _load_initial_level():
	load_level(starting_level, starting_spawn)
	
func load_level(scene_path: String, spawn_id: String):
	# Remove existing level if any
	if current_level:
		current_level.queue_free()
		current_level = null

	# Load new level
	var level_scene := load(scene_path)
	current_level = level_scene.instantiate()
	get_parent().add_child(current_level)

	position_player(spawn_id)


func position_player(spawn_id: String):
	var player := get_tree().get_first_node_in_group("player")
	var spawn := get_tree().get_first_node_in_group(spawn_id)
	var grid := get_tree().get_first_node_in_group("grid")
	
	if player and spawn and grid:
		player.initialize(grid, spawn.global_position)


func transition_to(target_scene: String, spawn_id: String):
	if transitioning:
		return

	transitioning = true
	await fade_out()

	get_tree().change_scene_to_file(target_scene)

	await get_tree().process_frame
	position_player(spawn_id)

	await fade_in()
	transitioning = false


func fade_out():
	# Placeholder – replace with real fade later
	await get_tree().create_timer(fade_time).timeout


func fade_in():
	await get_tree().create_timer(fade_time).timeout
