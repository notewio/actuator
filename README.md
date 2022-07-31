# actuator
Touchscreen gestures for Linux

## Usage
actuator looks for a config file at `$XDG_CONFIG_HOME/actuator/actuator.toml`.
It must contain the following values:

* `device_path = string` Path to touchscreen device
* `edge_tolerance = integer` How far away from the edge of the screen counts as the edge
* `min_distance = integer` How far you have to swipe to count as a gesture and not a tap
* `height & width = integer` Height and width of device, find with evtest.

## Actions
Actions are defined in the `[actions]` section of the config file, as `action_name = command`.
Each action is formatted by `{number of fingers}_{action type}`, e.g. `1_from_top`.

Possible types are `from_top`, `from_bottom`, `from_left`, `from_right`, `up`, `down`, `left`, `right`.


