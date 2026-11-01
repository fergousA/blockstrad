# Provenance and third-party notices

This is an unpublished local prototype, not an official blockst or Scratch
release. No endorsement by their authors is implied.

## blockst

The Rust parser, renderer, geometry, palettes and block/locale TOML tables under
`engine/` derive from **Loewe1000/blockst**, commit
`52b87da5619c89e6defe90b1142db36eaf3105ed` (repository version 0.3.0), with the local
French/Arabic colour fixes prepared in this workspace. The original MIT license
is reproduced in `licenses/blockst-MIT.txt` and applies to that code and data as
distributed by its author.

https://github.com/Loewe1000/blockst

blockstrad adds `engine/src/parser/translation.rs`, small request/ABI hooks and
the Typst frontend. The original blockst directory/package is not modified.
The engine is bundled because the current public blockst API neither exposes
all locale templates nor accepts a translated semantic AST for direct rendering.
A future upstream API could remove this vendoring requirement.

## Official Scratch menu strings

The additional contextual menu labels are extracted from the official
**scratchfoundation/scratch-l10n** snapshot
`0281712e1b8d94ac47b28075202294bb39db30be`, under **BSD-3-Clause** at that commit.
See `licenses/scratch-l10n-BSD.txt`, `scripts/menu-source.json` and the reproducible
build-time importer `scripts/generate-menus.py`.

https://github.com/scratchfoundation/scratch-l10n/tree/0281712e1b8d94ac47b28075202294bb39db30be

The snapshot is pinned deliberately: later scratch-l10n releases changed to
AGPL-3.0-only. No data from those later releases is included by this importer.
The menu snapshot is not claimed to be the latest live Scratch translation.
Block templates themselves remain those shipped by the pinned blockst engine.

## Original contributions

The new translation layer, tests and documentation were prepared with AI
assistance. Validation results describe commands actually run locally; no remote
release or CI execution is claimed. Maintainer identity and release metadata
should be reviewed before any public submission.
