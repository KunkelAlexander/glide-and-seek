extends Node2D

var grid: Node
@export var move_time := 0.15

@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D

var grid_pos: Vector2i
var spawn_world_pos: Vector2
var facing: Vector2i = Vector2i.DOWN

enum State {
	IDLE,
	MOVING,
	SLIDING
}

var state := State.IDLE
var slide_dir := Vector2i.ZERO


func _ready():
	update_animation(false)


func _process(_delta):
	if state != State.IDLE:
		return

	var dir := Vector2i.ZERO
	
	if Input.is_action_just_pressed("reset"): 
		initialize(grid, spawn_world_pos)

	if Input.is_action_pressed("ui_right"):
		dir = Vector2i.RIGHT
	elif Input.is_action_pressed("ui_left"):
		dir = Vector2i.LEFT
	elif Input.is_action_pressed("ui_down"):
		dir = Vector2i.DOWN
	elif Input.is_action_pressed("ui_up"):
		dir = Vector2i.UP

	if dir != Vector2i.ZERO:
		facing = dir
		attempt_move(dir)


func initialize(new_grid: Node, _spawn_world_pos: Vector2):
	grid = new_grid
	
	spawn_world_pos = _spawn_world_pos

	# Convert spawn WORLD position to GRID position
	grid_pos = grid.world_to_grid(spawn_world_pos)

	# Snap player to exact grid-aligned WORLD position
	global_position = grid.grid_to_world(grid_pos)
	
	print("Intialise player pos to ", grid_pos, " ", global_position)

	state = State.IDLE
	update_animation(false)
	
func attempt_move(dir: Vector2i):
	
	if grid == null or not is_instance_valid(grid):
		return
	var target := grid_pos + dir

	if grid.is_blocked(target):
		return

	if grid.is_ice(target):
		start_sliding(dir)
	else:
		move_one_tile(target)


func move_one_tile(cell: Vector2i):
	state = State.MOVING
	grid_pos = cell
	update_animation(true)

	var tween := create_tween()
	tween.tween_property(
		self,
		"global_position",
		grid.grid_to_world(grid_pos),
		move_time
	)

	tween.finished.connect(func():
		state = State.IDLE
		update_animation(false)
		check_for_exit()
	)


func start_sliding(dir: Vector2i):
	state = State.SLIDING
	slide_dir = dir
	update_animation(false)
	slide_step()


func slide_step():
	var next := grid_pos + slide_dir

	# Stop BEFORE moving if blocked
	if grid.is_blocked(next):
		state = State.IDLE
		update_animation(false)
		check_for_exit()
		return

	# Move into the next tile
	grid_pos = next

	var tween := create_tween()
	tween.tween_property(
		self,
		"global_position",
		grid.grid_to_world(grid_pos),
		move_time
	)

	tween.finished.connect(func():
		var following := grid_pos + slide_dir

		if grid.is_ice(grid_pos) and not grid.is_blocked(following):
			slide_step()
		else:
			state = State.IDLE
			update_animation(false)
			check_for_exit()
	)


func update_animation(moving: bool):
	if moving:
		match facing:
			Vector2i.UP:
				sprite.play("walk_up")
			Vector2i.DOWN:
				sprite.play("walk_down")
			Vector2i.LEFT:
				sprite.play("walk_left")
			Vector2i.RIGHT:
				sprite.play("walk_right")
	else:
		match facing:
			Vector2i.UP:
				sprite.play("idle_up")
			Vector2i.DOWN:
				sprite.play("idle_down")
			Vector2i.LEFT:
				sprite.play("idle_left")
			Vector2i.RIGHT:
				sprite.play("idle_right")

func check_for_exit():
	if not grid.is_exit(grid_pos):
		return

	var data = grid.get_exit_data(grid_pos)
	if data.is_empty():
		return

	var manager := get_tree().get_first_node_in_group("level_manager")
	if manager:
		print("Load level deferred with ", data.target, ", ", data.spawn)
		manager.load_level_deferred(data.target, data.spawn)
