# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added support for applying multiple style sheets to an entity [#45](https://github.com/afonsolage/bevy_ecss/issues/45)
- Added quality of life methods for modifying class lists on existing class components. [#48](https://github.com/afonsolage/bevy_ecss/pull/48)

### Changed

- Fixed wrong properties assignments [#42](https://github.com/afonsolage/bevy_ecss/pull/42/)
- Fixed bug with style sheets being applied in a non-deterministic order [#51](https://github.com/afonsolage/bevy_ecss/pull/51)

## [0.6.0]

### Added

- Added watch for changes on entities with components affected by css.

### Changed

- Fixed CSS precedence order, more broad rules are applied first.

## [0.5.1]

### Added

- Added support for `:hover` pseudo-class.
- Added support for [Pseudo-classes](https://developer.mozilla.org/en-US/docs/Web/CSS/Pseudo-classes).

## [0.5.1]

### Added

- New selectors for setting up BorderColor and Image texture.
- New documentation available on: [link](https://afonsolage.github.io/bevy_ecss/).

### Changed

- Updated `cssparser` to `0.33`.
- Use color function from `cssparser`.

## [0.5.0]

### Added

- Support for auto value

### Changed

- Upgraded to Bevy 0.12.

## [0.4.0]

### Changed

- Upgraded to Bevy 0.11.

## [0.3.0]

### Added

- `TextAlignProperty`

### Changed

- Upgraded to Bevy 0.10.

### Removed

- `VerticalAlignProperty` and `HorizontalAlignProperty`

## [0.2.0]

### Changed

- Upgraded to Bevy 0.9.

## [0.1.0]

### Added

- First version
