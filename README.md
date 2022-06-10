## Undergraduate Teaching Platform: TUI

[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fut-utp%2Ftui%2Fbadge&style=for-the-badge)](https://github.com/ut-utp/tui/actions) [![License: MPL-2.0](https://img.shields.io/github/license/ut-utp/tui?color=orange&style=for-the-badge)](https://opensource.org/licenses/MPL-2.0)
--
[![](https://tokei.rs/b1/github/ut-utp/tui)](https://github.com/ut-utp/tui) [![codecov](https://codecov.io/gh/ut-utp/tui/branch/main/graph/badge.svg)](https://codecov.io/gh/ut-utp/tui)

This repo is home to the [UTP `lc3-tui` crate](tui) as well as the binary targets that use the [`lc3-tui`](tui) crate.

Currently, we have the following binary targets:
  - [The desktop TUI application.](bins/tui.rs)
      + Runs on Windows, macOS, and Linux.
          * Note that the plugins feature is supported on x86-64 (and [aarch64](https://github.com/bytecodealliance/wasmtime/issues/3982)) platforms only due to [wasmtime](https://wasmtime.dev/)/[cranelift](https://github.com/bytecodealliance/cranelift) [restrictions](https://bytecodealliance.github.io/wasmtime/stability-platform-support.html) (TODO).
      + Currently has support for the following devices:
          * the [simulator](//github.com/ut-utp/prototype/blob/master/baseline-sim)
          * embedded devices w/UART based transports such as the [TI Launchpad](//github.com/ut-utp/tm4c)
  - [The web TUI application.](bins/web.rs)
      + TODO!
      + This supports all the functionality that the desktop application does, but supports different devices.

(TODO: move each target to their own crate? i.e. utp-tui, utp-web?)

![Root TUI Page](https://raw.githubusercontent.com/wiki/ut-utp/tui/root.png)


## Usage

TODO: link to manual/mdbook with detailed usage information

### Desktop App

Grab a release from the [Releases Page](https://github.com/ut-utp/tui/releases) and follow the installation instructions listed here.

TODO: run programs with `utp-tui /path/to/program`

TODO: connect to devices with `--device ...`
  - specific devices: `--device tm4c` (TODO: issue #6)
  - for boards, by serial port: `--device board=/dev/PORT:baud_rate`
  - etc.

You will need to flash your board the first time you use it with the TUI. In general this means running:
  - `utp-tui --flash <device>` (TODO: issue #10)

Some boards (like the TM4C) require [additional setup](). See the [list of supported boards](TODO: utp-devices repo!).

### Web App

Alternatively, ...

(TODO: link to the hosted web page version)

TODO: figure out what the story will be for webusb, webserial, etc. if we want to go that route.. (I think it's that webusb doesn't help us, webserial is sufficient for student use but is only support in Chrome and Edge)

<!-- move to UTP devices repo... -->
## Supported Boards

table with:
  -
