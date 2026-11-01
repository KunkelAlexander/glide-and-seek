extends Node


var audio_started := false

func _ready():
	set_process_input(true)

func _notification(what):
	# Pause the music while the app is in the background (e.g. minimised mobile browser)
	if what == NOTIFICATION_APPLICATION_FOCUS_OUT or what == NOTIFICATION_WM_WINDOW_FOCUS_OUT:
		$AudioStreamPlayer.stream_paused = true
	elif what == NOTIFICATION_APPLICATION_FOCUS_IN or what == NOTIFICATION_WM_WINDOW_FOCUS_IN:
		$AudioStreamPlayer.stream_paused = false

func _input(event):
	if audio_started:
		return

	if event is InputEventKey or event is InputEventMouseButton:
		audio_started = true
		
		var master := AudioServer.get_bus_index("Master")
		AudioServer.set_bus_mute(master, false)
		$AudioStreamPlayer.play()
