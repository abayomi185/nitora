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
