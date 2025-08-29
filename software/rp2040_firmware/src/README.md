# Files and what they do...

- animating_gif.rs:  Wrapper for the nostd animated gif library.  Displays on the 128x32 screen
- audio_playback.rs: Primary midi file to buffer interface
- backlight.rs:  Candidate LED driver.  Hardware needed for testing.
- button.rs:  Button interface.  Works.
- devices.rs: Container for the device abstractions.  Works
- display.rs: Wrapper for the display.  Works.
- led_driver.rs: User level LED setter.  Work in progress. 
- lib.rs: Files being compiled in.  Rust standard file.
- main.rs: The start point for the program.  Sets up the main menu
- menu.rs: Data driven menu system.  Works  
- piosound.rs:  Pulse sound output that interfaces with midi crate.  Hardware needed for testing

