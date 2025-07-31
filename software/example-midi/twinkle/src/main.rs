//! Play a sine wave for several seconds.
//!
//! A rusty adaptation of the official PortAudio C "paex_sine.c" example by Phil Burk and Ross
//! Bencina.

extern crate portaudio;
use midi_nostd::midi::Midi;
use midly::Smf;

use portaudio as pa;

const CHANNELS: i32 = 2;
const SAMPLE_RATE: f64 = 24_000.0;
const FRAMES_PER_BUFFER: u32 = 64;

fn midly_exploration() {
    let smf = Smf::parse(include_bytes!("../assets/vivaldi.mid")).unwrap();

    for (i, track) in smf.tracks.iter().enumerate() {
        println!("track {} has {} events", i, track.len());
        for event in track {
            println!("Event Detla {} {:?}", event.delta, event.kind);
        }
    }
}

fn main() {
    midly_exploration();

    match run() {
        Ok(_) => {}
        e => {
            eprintln!("Example failed with the following: {:?}", e);
        }
    }
}

fn get_loudest(smf: &Smf) -> i32 {
    println!("sampling midi");
    let mut midi = Midi::<48, 128, 32>::new(&smf, 1);
    let rval = midi.get_loudest_sample(smf);
    println!("done midi");
    rval
}

fn run() -> Result<(), pa::Error> {
    println!(
        "Midi STD Test.  SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let smf = midly::Smf::parse(include_bytes!("../assets/vivaldi.mid"))
        .expect("It's inlined data, so its expected to work");
    let loudest = get_loudest(&smf);
    println!("Loudest sample was {}", loudest);
    let mut midi = Midi::<24000, 128, 32>::new(&smf, (loudest / 0x8000) + 1);

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::StreamFlags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            let current = (midi.get_next(&smf).clip().to_i32() as f32) / 32768.0;
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

    println!("Test finished.");

    Ok(())
}
