extends Node2D

# The final ladder appears once every orb has been collected.
const LADDER := Vector2i(15, 23)


func _ready():
	var manager := get_tree().get_first_node_in_group("level_manager")
	if manager == null or manager.collected.size() < manager.TOTAL_STARS:
		return
	for dy in 2:
		$Logic.set_cell(LADDER + Vector2i(0, dy), 2, Vector2i(5, 1))
		for dx in 3:
			$Foreground.set_cell(LADDER + Vector2i(dx - 1, dy), 0, Vector2i(5 + dx, 13 + dy))
