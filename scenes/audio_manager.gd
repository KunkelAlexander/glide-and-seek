extends Node


var audio_started := false

func _ready():
	set_process_input(true)

func _input(event):
	if audio_started:
		return

	if event is InputEventKey or event is InputEventMouseButton:
		audio_started = true
		
		var master := AudioServer.get_bus_index("Master")
		AudioServer.set_bus_mute(master, false)
		$AudioStreamPlayer.play()
