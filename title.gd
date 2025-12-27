extends CanvasLayer

@export var hold_time := 5
@export var fade_time := 1.0

@onready var container: Control = $Control


func _ready():
	container.modulate.a = 1.0
	play()


func play():
	# Hold the title
	await get_tree().create_timer(hold_time).timeout

	# Fade out
	var tween := create_tween()
	tween.tween_property(
		container,
		"modulate:a",
		0.0,
		fade_time
	)
	await tween.finished

	# Hide completely so it doesn't block clicks
	visible = false
