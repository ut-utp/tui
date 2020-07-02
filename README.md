## Undergraduate Teaching Platform: TUI

[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fut-utp%2Ftui%2Fbadge&style=for-the-badge)](https://github.com/ut-utp/tui/actions) [![License: MPL-2.0](https://img.shields.io/github/license/ut-utp/tui?color=orange&style=for-the-badge)](https://opensource.org/licenses/MPL-2.0)
--
[![](https://tokei.rs/b1/github/ut-utp/tui)](https://github.com/ut-utp/tui) [![codecov](https://codecov.io/gh/ut-utp/tui/branch/main/graph/badge.svg)](https://codecov.io/gh/ut-utp/tui)

This repo is home to the [UTP `lc3-tui` crate](tui) as well as the binary targets that use the [`lc3-tui`](tui) crate.

Currently, we have the following binary targets:
  - [The desktop TUI application.](bins/tui.rs)
      + Runs on Windows, macOS, and Linux.
          * Note that the plugins feature is supported on x86-64 platforms only due to [wasmtime](https://wasmtime.dev/)/[cranelift](https://github.com/bytecodealliance/cranelift) [restrictions](https://bytecodealliance.github.io/wasmtime/stability-platform-support.html) (TODO).
      + Currently has support for the following devices:
          * the [simulator](//github.com/ut-utp/prototype/blob/master/baseline-sim)
          * embedded devices w/UART based transports such as the [TI Launchpad](//github.com/ut-utp/tm4c)
  - [The web TUI application.](bins/web.rs)
      + TODO!
      + This supports all the functionality that the desktop application does, but supports different devices.

(TODO: move each target to their own crate? i.e. utp-tui, utp-web?)

![Root TUI Page](https://raw.githubusercontent.com/wiki/ut-utp/tui/root.png)
