extends Node

@export var fade_time := 1.0

var transitioning := false

@export var starting_level := "res://scenes/level_1.tscn"
@export var starting_spawn := "entrance_a"
@onready var fade_rect: ColorRect = $"../Fade/FadeRect"


var current_level: Node = null

const TOTAL_STARS := 5
var collected := {}
@onready var star_label: Label = $"../Stars/Label"


var start_msec := -1
var won := false
var stats := {"moves": 0, "tiles": 0, "longest": 0, "resets": 0}
@onready var reward: CanvasLayer = $"../Reward"
@onready var reward_time: Label = $"../Reward/Center/Box/Time"


func start_timer():
	if start_msec < 0:
		start_msec = Time.get_ticks_msec()


func win():
	var seconds := (Time.get_ticks_msec() - start_msec) / 1000
	reward_time.text = "%d min %02d s" % [seconds / 60, seconds % 60]
	reward.get_node("Center/Box/Stats").text = "
".join([
		"%d key presses, %d tiles of glide" % [stats.moves, stats.tiles],
		"Longest slide: %d tiles. Wheee." % stats.longest,
		"That is %d penguins laid end to end" % stats.tiles,
		"Times you hit RESET: %d%s" % [stats.resets, " (flawless!)" if stats.resets == 0 else ""],
		"Orbs collected: %d/%d" % [collected.size(), TOTAL_STARS],
	])
	won = true
	get_tree().get_first_node_in_group("player").grid = null  # freeze input behind the reward screen
	reward.visible = true
	await get_tree().create_timer(1.5).timeout  # ignore the keys still held from the last slide
	reward.get_node("Center/Box/Hint").visible = true


func _input(event):  # before the GUI, so the reward overlay cannot swallow a tap
	if won and reward.get_node("Center/Box/Hint").visible and (
			(event is InputEventKey and event.pressed) or (event is InputEventScreenTouch and event.pressed)
			or (event is InputEventMouseButton and event.pressed)):
		restart()


func restart():
	won = false
	start_msec = -1
	stats = {"moves": 0, "tiles": 0, "longest": 0, "resets": 0}
	collected.clear()
	star_label.text = "ORBS 0/%d" % TOTAL_STARS
	reward.visible = false
	reward.get_node("Center/Box/Hint").visible = false
	load_level_deferred(starting_level, starting_spawn)


func collect_star(id: String):
	collected[id] = true
	star_label.text = "ORBS %d/%d" % [collected.size(), TOTAL_STARS]


func _ready():
	# Defer loading until the scene tree is fully ready
	load_level_deferred(starting_level, starting_spawn)

func load_level_deferred(scene_path: String, spawn_id: String):
	call_deferred("load_level", scene_path, spawn_id)
	
func load_level(scene_path: String, spawn_id: String):
	# Disable player while loading a new level
	var player := get_tree().get_first_node_in_group("player")
	if player:
		player.set_process(true)
		player.grid = null


	await fade_out()



	# Remove old level
	if current_level and is_instance_valid(current_level):
		current_level.queue_free()
		current_level = null
		

	# IMPORTANT: let the old level fully exit
	await get_tree().process_frame

	# Load new level
	var level_scene := load(scene_path)
	if level_scene == null:
		push_error("LevelManager: Failed to load level scene: " + scene_path)
		return
		
	current_level = level_scene.instantiate()
	get_parent().add_child(current_level)


	# IMPORTANT: wait for Grid + Spawn to be ready
	await get_tree().process_frame
	
	position_player(spawn_id)


	await fade_in()

	# Reenable player
	if player:
		player.set_process(true)


func position_player(spawn_id: String):
	var player := get_tree().get_first_node_in_group("player")
	var spawn := get_tree().get_first_node_in_group(spawn_id)
	var grid := get_tree().get_first_node_in_group("grid")
	
	
	if not player:
		push_error("LevelManager: Player not found")
		return
	if not grid:
		push_error("LevelManager: Grid not found")
		return
	if not spawn:
		push_error("LevelManager: Spawn not found: " + spawn_id)
		return
		
	player.initialize(grid, spawn.global_position)

func fade_out():
	fade_rect.visible = true

	var tween := create_tween()
	tween.tween_property(
		fade_rect,
		"modulate:a",
		1.0,
		fade_time
	)

	await tween.finished


func fade_in():
	var tween := create_tween()
	tween.tween_property(
		fade_rect,
		"modulate:a",
		0.0,
		fade_time
	)

	await tween.finished
	fade_rect.visible = false
