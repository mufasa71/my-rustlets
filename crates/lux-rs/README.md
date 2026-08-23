# Lux Sensor System Color Mode

This is a simple `systemd` compatible daemon for toggle the color mode with `darkman`.
It allows you to set the color mode to either "light" or "dark" based on your preferences.

## Prerequisites

- [darkman](https://darkman.whynothugo.nl/): command-line tool and daemon to
  control the color mode of your system
- [i3status-rust](https://github.com/greshake/i3status-rust): fast and
  lightweight status bar for i3 window manager
- Lux Sensor: `zigbee2mqtt` based sensor that can detect ambient light levels
  and provide data to your system
