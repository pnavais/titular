# Titular

A command-line tool to display fancy titles in your terminal with syntax highlighting and theme support.

![Titular Demo](assets/demo.png)

## Features

- 🎨 Syntax highlighting for various file formats
- 🌈 Support for multiple color themes
- 📝 Customizable title templates
- 🔄 Real-time title updates
- 🎯 Multiple output formats (ANSI, HTML, etc.)
- ⚡ Fast and efficient processing

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/pnavais/titular.git
cd titular

# Build with default features
cargo build --release

# Install
cargo install --path .
```

### From Cargo

```bash
cargo install titular
```

## Usage

```bash
# Display a title with default settings
titular display "Hello, World!"

# Use a specific theme
titular display --theme dracula "Hello, World!"

# Display a file with syntax highlighting
titular display --file example.rs

# List available themes
titular themes

# Create a new title template
titular template create my-template
```

## Features

Titular comes with different feature sets that can be enabled during installation:

- `minimal`: Basic functionality with terminal size detection
- `application`: Default feature set including minimal and fetcher
- `full_application`: All features including display capabilities
- `display`: Syntax highlighting and theme support
- `display-themes`: Extended theme support

To install with specific features:

```bash
cargo install titular --features full_application
```

## Themes

Titular supports a variety of themes:

### Base Themes

- Catppuccin
- Dracula

### Extended Themes

- Ayu
- Dark Material
- Darkula
- Enki
- Gruvbox
- Monokai
- Monokai++
- Nord
- OneHalf
- Solarized

## Configuration

Titular can be configured through:

1. Command-line arguments
2. Configuration file (`~/.config/titular/config.toml`)
3. Environment variables

Example configuration:

```toml
[display]
theme = "dracula"
syntax = "rust"
format = "ansi"
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding New Themes

1. Add the theme as a git submodule in `assets/themes/`:

   ```bash
   git submodule add <theme-repo-url> assets/themes/<theme-name>
   ```

2. Update the build script to include the new theme

3. Submit a pull request with your changes

## License

This project is dual-licensed under both the MIT License and the Apache License 2.0. You may choose either license at your option.

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

### Why Dual Licensing?

The dual MIT/Apache 2.0 licensing provides:

- **Maximum Flexibility**: Users can choose which license terms they prefer
- **Patent Protection**: Apache 2.0 provides explicit patent protection
- **Ecosystem Alignment**: Aligns with Rust's ecosystem standards
- **Compatibility**: Covers both GPLv3 compatibility and maximum permissiveness

## Acknowledgments

- [Bat](https://github.com/sharkdp/bat) - The main source of inspiration for this project, an outstanding cat clone with wings
- [Syntect](https://github.com/trishume/syntect) for syntax highlighting
- All theme creators for their amazing color schemes
- The Rust community for their excellent tools and libraries

## Author

Pablo Navais - [@pnavais](https://github.com/pnavais)
