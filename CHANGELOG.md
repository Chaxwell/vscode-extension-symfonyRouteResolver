# Change Log

## [0.9.0] - 2026-02-28

### Added

- Clickable document links for Symfony route names in PHP, Twig, and YAML files — clicking a route name opens the corresponding PHP controller at the exact declaration line.
- Automatic indexing at workspace startup using `symfony console debug:router`.
- Automatic re-indexing on PHP file save (debounced 1.5 s).
- Manual re-index command: `Symfony Route Resolver: Re-index routes` (Command Palette).
- Status bar message showing the number of indexed routes after each indexation.
- Rust sidecar process (`symfony-route-resolver-sidecar`) handling indexing, PHP AST parsing (via `mago-syntax`), and multi-pattern text search (via AhoCorasick).
- `symfony-route-resolver.symfonyBinaryPath` setting to configure a custom path to the Symfony CLI binary.
- Cross-platform build scripts targeting Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64.
