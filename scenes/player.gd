extends Node2D

@export var grid: Node
@export var move_time := 0.15

var grid_pos: Vector2i

enum State {
	IDLE,
	MOVING,
	SLIDING
}

var state := State.IDLE
var slide_dir := Vector2i.ZERO


func _ready():
	assert(grid != null, "Player: Grid not assigned!")
	grid_pos = grid.world_to_grid(global_position)
	global_position = grid.grid_to_world(grid_pos)
	queue_redraw()

func _process(_delta):
	if state != State.IDLE:
		return

	var dir := Vector2i.ZERO

	if Input.is_action_just_pressed("ui_right"):
		dir = Vector2i.RIGHT
	elif Input.is_action_just_pressed("ui_left"):
		dir = Vector2i.LEFT
	elif Input.is_action_just_pressed("ui_down"):
		dir = Vector2i.DOWN
	elif Input.is_action_just_pressed("ui_up"):
		dir = Vector2i.UP

	if dir != Vector2i.ZERO:
		attempt_move(dir)


func attempt_move(dir: Vector2i):
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

	var tween := create_tween()
	tween.tween_property(
		self,
		"global_position",
		grid.grid_to_world(grid_pos),
		move_time
	)
	tween.finished.connect(func():
		state = State.IDLE
		queue_redraw()
	)

func start_sliding(dir: Vector2i):
	state = State.SLIDING
	slide_dir = dir
	slide_step()

func slide_step():
	var next := grid_pos + slide_dir

	# Stop BEFORE moving if blocked
	if grid.is_blocked(next):
		state = State.IDLE
		queue_redraw()
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
		# Decide if we can continue sliding
		var following := grid_pos + slide_dir

		if grid.is_ice(grid_pos) and not grid.is_blocked(following):
			slide_step()
		else:
			state = State.IDLE
			queue_redraw()
	)



func _draw():
	draw_circle(Vector2.ZERO, 12, Color.BLACK)
