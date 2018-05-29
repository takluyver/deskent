Linux desktops use desktop entry files to find and launch graphical applications.
The launcher or menu is populated with names and icons from these files.

`deskent` is a command line tool to find and inspect desktop entry files.

## Installation

```
cargo install deskent
```

`cargo` is Rust's package manager - see https://www.rust-lang.org/

## Usage

```shell
# List all desktop entry files
deskent ls

# Find files by display name (case insensitive)
deskent find firefox
```
