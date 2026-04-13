# Nitora

Nitora is a macOS CLI for controlling extended XDR/EDR brightness.

It uses:

- a tiny AppKit + Metal overlay to trigger EDR/XDR mode
- CoreGraphics gamma table control to adjust brightness

> [!NOTE]
> This project was built with the help of AI.

## Installation

### Homebrew

```bash
brew tap abayomi185/tap
brew install nitora
```

### From source

```bash
cargo install --git https://github.com/abayomi185/nitora.git
```

## Commands

```bash
nitora serve
nitora enable
nitora disable
nitora toggle
nitora status
nitora set 42
```

## Development

Build locally:

```bash
cargo build
```

Run the service:

```bash
cargo run -- serve
```

In another terminal:

```bash
cargo run -- status
cargo run -- enable
cargo run -- set 60
cargo run -- disable
```

## Nix

This repo includes:

- a flake package
- a dev shell
- a nix-darwin module

Build with Nix:

```bash
nix build .#nitora
```

Run the CLI:

```bash
nix run .#nitora -- --help
```

Enter the dev shell:

```bash
nix develop
```

### nix-darwin

```nix
{
  imports = [ inputs.nitora.darwinModules.default ];

  programs.nitora = {
    enable = true;
    autoActivate = true;
  };
}
```

## Inspiration / Similar projects

- [BrightIntosh](https://github.com/niklasr22/BrightIntosh) — a macOS utility for unlocking higher XDR display brightness.
- [BrightXDR](https://github.com/starkdmi/BrightXDR) — an open-source proof-of-concept for XDR/HDR extra brightness on macOS.
