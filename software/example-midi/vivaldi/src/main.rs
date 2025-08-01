//! Midi playback example using PortAudio

extern crate portaudio;
use midi_nostd::midi::Midi;

use portaudio as pa;

const CHANNELS: i32 = 2;
const SAMPLE_RATE: f64 = 24_000.0;
const FRAMES_PER_BUFFER: u32 = 64;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn run() -> Result<(), pa::Error> {
    println!(
        "Midi Playback Example.  SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let (header, tracks) = midly::parse(include_bytes!("../assets/vivaldi.mid"))
        .expect("It's inlined data, so its expected to parse");

    type MyMidi<'a> = Midi<'a, 24000, 64, 32>;
    println!(
        "Midi structure is currently using {} bytes",
        std::mem::size_of::<MyMidi>()
    );

    let mut midi = MyMidi::new(&header, tracks);

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::StreamFlags::CLIP_OFF;

    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let current = (midi.get_next().clip().to_i32() as f32) / 32768.0;
            buffer[idx] = current;
            buffer[idx + 1] = current;
            idx += 2;
        }
        if midi.has_next() {
            pa::Continue
        } else {
            pa::Complete
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;
    while stream.is_active()? {
        pa.sleep(100);
    }
    stream.stop()?;
    stream.close()?;

    println!("Midi Playback Example Finished.");

    Ok(())
}
