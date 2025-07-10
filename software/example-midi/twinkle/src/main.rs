//! Play a sine wave for several seconds.
//!
//! A rusty adaptation of the official PortAudio C "paex_sine.c" example by Phil Burk and Ross
//! Bencina.

extern crate portaudio;

use portaudio as pa;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 24_000.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 200;

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
        "Midi STD Test.  SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let mut wave_pool = midi_nostd::wave_generator::WavePool::new();
    let mut all_pools = midi_nostd::sound_sources::SoundSources::<midi_nostd::sound_sample::SoundSampleI32, 24000>::create_with_single_pool_for_test(
        &mut wave_pool,
        midi_nostd::sound_source_id::SoundSourceType::WaveGenerator,
    );
    let wave_id = all_pools.alloc(midi_nostd::sound_source_id::SoundSourceType::WaveGenerator);
    midi_nostd::wave_generator::set_wave_properties(
        &mut all_pools,
        &wave_id,
        midi_nostd::sound_source_msgs::WaveType::Sine,
        2600,
        25,
        100,
    );
    let _grouping = ( all_pools, wave_pool );

    // Initialise sinusoidal wavetable.
    let mut left_phase = 0;
    let mut right_phase = 0;

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
        let mut _new_msgs = midi_nostd::sound_source_msgs::SoundSourceMsgs::default();
        for _ in 0..frames {
            //all_pools.update(&mut new_msgs);
            //let current = groupingall_pools.get_next(&wave_id);
            buffer[idx] = 0; //sine[left_phase];
            buffer[idx + 1] = 0; // = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 3;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            idx += 2;
        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    println!("Test finished.");

    Ok(())
}
