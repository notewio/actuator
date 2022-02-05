# actuator
Touchscreen gestures for Linux

## Usage
actuator looks for a config file at `$XDG_CONFIG_HOME/actuator/Actuator.toml`. It must contain the following values:

* `device_path = string` Path to touchscreen device
* `edge_tolerance = integer` How far away from the edge of the screen counts as the edge
* `min_distance = integer` How far you have to swipe to count as a gesture and not a tap
* `screen_height & screen_width = integer` Height and width of screen, find with evtest.

## Actions
Actions are defined in the `[actions]` section of the config file, as `action_name = command`. Available actions are:

* 1\_from\_top
* 1\_from\_bottom
* 1\_from\_left
* 1\_from\_right
* 2\_pinch
* 2\_spread
* 2\_up
* 2\_down
* 2\_left
* 2\_right
