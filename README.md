
# Cargo Lock 2 Rpm Provides

Fedora requires you to declare all bundled/vendored dependencies in RPMS. To help make this
less painful, this tool allows generating that list for you to copy to your rpm.spec file.

## Installation

```
cargo install cargo-lock2rpmprovides
```

## Usage

From your Rust/Cargo project directory run:

```
cargo lock2rpmprovides
```

For debug run with:

```
cargo lock2rpmprovides -d
```

To use a different directory:

```
cargo lock2rpmprovides /folder/that/contains/rust/project
```




