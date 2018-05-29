Linux desktops use desktop entry files to find and launch graphical applications.
The launcher or menu is populated with names and icons from these files.

`deskent` is a command line tool to find and inspect these files.

Installation:

```
cargo install deskent
```

Usage:

```shell
# List all desktop entry files
deskent ls

# Find files by display name (case insensitive)
deskent find firefox
```
