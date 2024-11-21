# RPI hacker newyear prototyping project

Prototyping for the RPI Pico (rp2024) software for the Hacker New Year 2024 art project.

## Building Instructions

### Installing Rust

Follow instructions here [https://www.rust-lang.org/tools/install]

### Getting a rp2024 cross compiling environment working

```console
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

### Building and running

cargo run

The current configuartion is not setup for debugging - the executable image
is downloaded directly into the Pico.

### Committing with checks

A pre-commit tool has been added to keep the repository clean.  To run the
pre-commit tool, install it by following the following instructions [https://pre-commit.com/]
then type:

```console
pre-commit install
```
