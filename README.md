# Compile Commander

A simple command line utility to make small adjustments to clang compilation databases. Currently useful for adding & removing include directories that were erroneously added/omitted by build tools (common in embedded development).

## Usage

```
compile-commander --help
```

lists all possible operations in detail. If you just want to add an include directory, the following command works:

```
compile-commander -i "/usr/include"
```


## Building

After cloning:

```
cd compile-commander
cargo build
cargo install --path .
```
:)
