# Symfony Route Resolver

A VS Code extension that turns every Symfony route name found in your files into a clickable link, navigating directly to the corresponding PHP controller.

## How it works

When you open a PHP, Twig, or YAML file, the extension automatically detects route names surrounded by quotes (e.g. `'app_home'`, `"user_profile"`) and makes them clickable. A click opens the controller file at the exact line of the method or class annotated with that route.

```twig
{# Click on 'app_home' to jump to HomeController.php line 23 #}
{{ path('app_home') }}
```

Indexing relies on `symfony console debug:router` to list all routes, then on PHP AST analysis to pinpoint their declarations.

## Requirements

- VS Code ≥ 1.95.0
- [Symfony CLI](https://symfony.com/download) available in `$PATH` (or configured path)
- A Symfony project open as a workspace

## Installation

*More informations to come about installation*

1. Download the `.vsix` file for your platform.
2. In VS Code: `Extensions` → `...` → `Install from VSIX…`
3. Select the downloaded `.vsix` file.

Or from the command line:

```bash
code --install-extension symfony-route-resolver-0.1.0-linux-x64.vsix
```

## Configuration

| Setting | Type | Default | Description |
|---|---|---|---|
| `symfony-route-resolver.symfonyBinaryPath` | `string` | `symfony` | Path to the Symfony CLI binary |

Example `.vscode/settings.json`:

```json
{
    "symfony-route-resolver.symfonyBinaryPath": "/usr/local/bin/symfony"
}
```

## Usage

The extension works entirely automatically:

- **On startup** — routes are indexed as soon as the workspace is opened.
- **On every `.php` file save** — re-indexing is triggered automatically (1.5 s debounce to avoid repeated runs).
- **Manually** — via the Command Palette (`Ctrl+Shift+P`): `Symfony Route Resolver: Re-index routes`.

The number of indexed routes is briefly shown in the status bar (`Symfony Routes: 42 routes indexed`).

## Supported file types

Links are activated in the following languages:

- PHP (`.php`)
- Twig (`.twig`)
- YAML (`.yaml`, `.yml`)

## Technical architecture

The extension is built on two components:

- **TypeScript extension** — handles VS Code integration (document links, commands, file watchers).
- **Rust sidecar** (`bin/symfony-route-resolver-sidecar`) — performs indexing via `debug:router`, PHP AST analysis (via `mago-syntax`), and multi-pattern text search (via AhoCorasick). Both processes communicate over stdin/stdout using NDJSON.

## Building from source

```bash
# Install Node dependencies
npm install

# Compile the Rust sidecar (Linux x64)
npm run compile-rust:linux

# Other available targets
npm run compile-rust:linux-arm   # Linux ARM64
npm run compile-rust:mac         # macOS Intel
npm run compile-rust:mac-arm     # macOS Apple Silicon
npm run compile-rust:windows     # Windows x64

# Package the extension
npm run package:linux

# Run Rust tests
npm run test:rust
```

> Cross-compilation requires [`cross`](https://github.com/cross-rs/cross) and Docker.
> Without cross, install the target toolchain via `rustup target add <triple>`.
