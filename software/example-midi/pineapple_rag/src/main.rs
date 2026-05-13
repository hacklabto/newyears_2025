//! Midi playback example using PortAudio

use midi_nostd::midi::Midi;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bring up midi player
    let (header, tracks) = midly::parse(include_bytes!("../assets/pineapple_rag.mid"))
        .expect("It's inlined data, so its expected to parse");

    type MyMidi<'a> = Midi<'a, 44100, 441, 64, 32>;

    println!(
        "Midi structure is currently using {} bytes",
        std::mem::size_of::<MyMidi>()
    );

    let mut midi = MyMidi::new(&header, tracks);
    
    // 1. Initialize the audio host (ALSA on Ubuntu)
    let host = cpal::default_host();

    // 2. Get the default output device
    let device = host.default_output_device()
        .ok_or("No output device found")?;

    // 3. Get the default configuration
    let config = device.default_output_config()?;
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    // Make sure the sample rate is 44100, which is what I set the
    // midi player to.  Knowing values like sample rate at compile
    // time helps keep the size of the midi library small (10400
    // bytes) and helps Rust optimize playback, but the cost is needing
    // to know the playback rate in advance.
    //
    println!("Playing on: {} Sample rate {}", device.name()?, sample_rate);
    assert!( sample_rate == 44100.0 );

    // 5. Build the output stream
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
 
    // Shared state: 'false' means still playing, 'true' means source exhausted
    let finished = Arc::new(AtomicBool::new(false));
    let finished_for_callback = finished.clone();

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = (midi.get_next().clip().to_i32() as f32) / 32768.0;
                for sample in frame.iter_mut() {
                    *sample = value;
                }
                if !midi.has_next() {
                    finished_for_callback.store(true, Ordering::Relaxed);
                }
            }
        },
        err_fn,
        None,
    )?;

    // 6. Start the stream and wait
    stream.play()?;
    while !finished.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(100));
    }
    stream.pause()?;

    Ok(())
}
