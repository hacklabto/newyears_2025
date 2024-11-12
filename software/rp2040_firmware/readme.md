# RPI hacker newyear prototyping project.

Prototyping for the RPI Pico software for the Hacker New Year 2024 art project.

## Building Instructions

Starting with a Linux rust setup...

```
rustup target add thumbv6m-none-eabi
cargo run
```

The current configuartion is not setup for debugging - the executable image
is downloaded directly into the Pico.

