# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/mufasa71/my-rustlets/releases/tag/lux-rs-v0.1.0) - 2026-10-28

### Added

- *(lux-rs)* use common mqtt-to-dbus library ([#33](https://github.com/mufasa71/my-rustlets/pull/33))
- *(cli)* add max_illuminance, light_threshold and dark_threshold options ([#26](https://github.com/mufasa71/my-rustlets/pull/26))
- *(mqtt)* use UUID v4 as dynamic client identifier ([#25](https://github.com/mufasa71/my-rustlets/pull/25))
- *(cli)* implement Clap parser for runtime configuration

### Fixed

- *(sensor)* update MAX_ILLUMINANCE to match observed sensor range
- *(sensor)* correct MAX_ILLUMINANCE value from 338.0 to 419.0

### Other

- *(config)* remove obsolete fnox local example files
- *(config)* add example  with placeholder secrets
- *(main)* replace command‑line topic argument with environment variable
- *(main)* add TODO comment for configurable MQTT host and port
- *(sensor)* replace hard‑coded TOPIC with runtime argument
- *(sensor)* document new LIGHT_THRESHOLD, DARK_THRESHOLD, and TOPIC constants ([#19](https://github.com/mufasa71/my-rustlets/pull/19))
- Next ([#18](https://github.com/mufasa71/my-rustlets/pull/18))
