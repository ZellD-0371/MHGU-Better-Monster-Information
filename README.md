<p align="center">
  <strong>English</strong> | <a href="README_ko.md">한국어</a>
</p>

<h3 align="center">Screenshots</h3>

<p align="center">
  <a href="docs/screenshots.md">View screenshots</a>
</p>

# MHGU Better Monster Information

A Forge mod that adds a monster information page to the Hunter's Notes in Monster Hunter Generations Ultimate. View hitzones, body part diagrams, status ailment resistance, trap effectiveness, and material acquisition information directly in the game.

## Supported environments

- MHGU Global version 1.4.0
- MHGU Recomp

Emulators are currently unsupported.

## Features

- Adds an information page to the end of each registered small and large monster's entry
- Highlights the selected hitzone on the body part diagram using circles or ellipses
- Displays status ailment resistance and trap effectiveness using database values
- Shows material drop rates and detailed acquisition conditions for Low Rank, High Rank, G Rank, and Deviant quest tiers

## Installation

1. Close the game.
2. Extract `Better-Monster-Information_vX.X.X.zip`.
3. Place the `Better-Monster-Information` folder inside the `mods` folder next to the Recomp executable.
4. Launch the game.

```text
mods/
└─ Better-Monster-Information/
   └─ romfs/
      └─ nativeNX/
         └─ plugins/
            ├─ better_monster_information.nro
            └─ better-monster-information/
               ├─ monster-icons.rgba
               ├─ languages/
               │  ├─ text.en.bin … text.es.bin
               │  └─ font.shared.otf
               └─ parts/
```

### Standard / Low Memory

- Standard: `Better-Monster-Information_vX.X.X.zip` — original icon atlas and full-resolution body diagrams.
- Low Memory: `Better-Monster-Information_LowMemory_vX.X.X.zip` — icons capped at 32px in a 2MiB atlas; diagrams scaled at runtime to a maximum 384×256 canvas.

Install only one version. Both use the same mod folder and settings; close the game before replacing the folder. 

## Usage

### Viewing monster information

Open the Hunter's Notes monster list from the game menu and select a registered monster. Use R to navigate to the final page to view the additional information.

## Settings

Press Insert to open the Forge menu, then navigate to `Plugin UI > Better Monster Information`.

| Setting | Description |
| --- | --- |
| Language | Select from 9 supported languages. Default: English |
| Button layout | Nintendo / Xbox / PlayStation. Default: Nintendo |

Supported languages: English, 简体中文, 繁體中文, 日本語, 한국어, Deutsch, Français, Italiano, Español

## Save data

| File | Contents |
| --- | --- |
| `better-monster-information.ini` | Global language and button layout settings |

The default Windows Recomp save location is `%APPDATA%\MHGURecomp\save\save_0\`. If your environment uses a custom save location, that location is used instead.

## Building from source

Required tools: Rust nightly with rust-src, the Forge Cargo build tool (`cargo forge`), devkitARM, and Python 3.10 or later. The Forge SDK commit is specified in `Cargo.toml`.

```powershell
./tools/Build.ps1
./tools/Test.ps1
py -3.10 -X utf8 tools/package_native.py
```

The outputs are `better_monster_information.nro`, `dist/native/Better-Monster-Information`, and `dist/Better-Monster-Information_vX.X.X.zip`. Check the devkitARM path in the build script and adjust it for your environment.

Automated tests do not replace in-game input and visual testing.


```powershell
# Regenerate changed text/font inputs with fontTools.
py -3.10 -X utf8 tools/prepare_languages.py
# Generate the small atlas only when icon inputs change (requires Pillow).
py -3.10 -X utf8 tools/prepare_low_memory.py
./tools/Build.ps1 -Variant low-memory
./tools/Test.ps1 -Variant low-memory
py -3.10 -X utf8 tools/package_native.py --variant low-memory
```

## Repository structure

`src`, `.cargo`, `link`, `conf`, `assets/native`, and `tools` contain the files required to build, test, and package the native mod. Keep `Cargo.lock`, the generated Rust data, and the native resources alongside them.

## Sources and credits

The body part diagrams and hitzone/material data come from the [MHXX Dex web port](https://mhxx-dex.pages.dev/), based on Ping’s MHXX Dex.

- **Ping** — original MHXX Dex
- **fabric_soul** — MHXX Dex web port
- Contributors credited by the web port: breakds, 掌机迷, MHVuze, svanheulen, ACG汉化组, dasding and team, 팀 갈무리, Zeff and team. This list does not establish authorship of each individual diagram.
- **mhgu-forge / forge-rs** — Fexty, MIT License
