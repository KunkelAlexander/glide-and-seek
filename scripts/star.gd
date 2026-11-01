extends Sprite2D

# Collectible star. Collected stars stay gone when the level is reloaded.
@export var star_id := ""


func _ready():
	texture = preload("res://assets/star.png")
	_add_shadow()
	var manager := get_tree().get_first_node_in_group("level_manager")
	if manager and manager.collected.has(star_id):
		queue_free()


# Flat grey ellipse under the orb, like the rocks in the tileset, so it sits on the ice.
func _add_shadow():
	var points := PackedVector2Array()
	for i in 24:
		var a := TAU * i / 24.0
		points.append(Vector2(cos(a) * 11.0, 11.0 + sin(a) * 4.0))
	var shadow := Polygon2D.new()
	shadow.polygon = points
	shadow.color = Color(0.62, 0.68, 0.74, 0.8)
	shadow.show_behind_parent = true
	add_child(shadow)
