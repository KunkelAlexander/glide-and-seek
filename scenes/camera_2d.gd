extends Camera2D

@export var tile_size: int = 32
@export var tiles_on_long_side: int = 30
@export var debug_camera_zoom: bool = false

var _last_viewport_size: Vector2i = Vector2i.ZERO

func _ready():
	# Connect to viewport resize
	get_viewport().size_changed.connect(_on_viewport_size_changed)
	
	# Initial calculation
	update_camera_zoom("initial")

func _on_viewport_size_changed():
	update_camera_zoom("viewport resized")

func update_camera_zoom(reason: String = "unknown"):
	var viewport_size: Vector2i = get_viewport_rect().size

	# Avoid redundant recalculations
	if viewport_size == _last_viewport_size:
		return
	_last_viewport_size = viewport_size

	var long_side_pixels: int = max(viewport_size.x, viewport_size.y)
	var desired_world_pixels: int = tile_size * tiles_on_long_side
	var zoom_value: float = 1 / (float(desired_world_pixels) / float(long_side_pixels))

	zoom = Vector2(zoom_value, zoom_value)

	if debug_camera_zoom:
		print("================ Camera Zoom Update ================")
		print("Reason: ", reason)
		print("Viewport size: ", viewport_size)
		print("Tile size: ", tile_size)
		print("Tiles on long side: ", tiles_on_long_side)
		print("Desired world pixels: ", desired_world_pixels)
		print("Long screen side (px): ", long_side_pixels)
		print("Computed zoom value: ", zoom_value)
		print("Applied zoom: ", zoom)
		print("===================================================")
