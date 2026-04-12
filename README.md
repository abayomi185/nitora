# Nitora

Nitora is a macOS CLI for controlling extended XDR/EDR brightness.

It uses:

- a tiny AppKit + Metal overlay to trigger EDR/XDR mode
- CoreGraphics gamma table control to adjust brightness

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
- a Home Manager module

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

  programs.nitora.enable = true;
}
```

### Home Manager

```nix
{
  imports = [ inputs.nitora.homeManagerModules.default ];

  programs.nitora.enable = true;
}
```
