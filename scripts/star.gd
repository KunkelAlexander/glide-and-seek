extends Sprite2D

# Collectible star. Collected stars stay gone when the level is reloaded.
@export var star_id := ""


func _ready():
	texture = preload("res://assets/star.png")
	var manager := get_tree().get_first_node_in_group("level_manager")
	if manager and manager.collected.has(star_id):
		queue_free()
