# RPI hacker newyear prototyping project.

Prototyping for the RPI Pico (rp2024) software for the Hacker New Year 2024 art project.

## Building Instructions

### Installing Rust

Follow instructions here

https://www.rust-lang.org/tools/install

### Getting a rp2024 cross compiling environment working.

```
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```

### Building and running
cargo run

The current configuartion is not setup for debugging - the executable image
is downloaded directly into the Pico.

