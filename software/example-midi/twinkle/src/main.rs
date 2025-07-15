//! Play a sine wave for several seconds.
//!
//! A rusty adaptation of the official PortAudio C "paex_sine.c" example by Phil Burk and Ross
//! Bencina.

extern crate portaudio;
use midi_nostd::adsr::create_adsr;
use midi_nostd::amp_mixer::create_amp_mixer;
use midi_nostd::sound_sample::SoundSample;
use midi_nostd::sound_sample::SoundScale;
use midi_nostd::sound_source_msgs::SoundSourceAdsrInit;
use midi_nostd::sound_source_msgs::SoundSourceAmpMixerInit;
use midi_nostd::sound_source_msgs::SoundSourceKey;
use midi_nostd::sound_source_msgs::SoundSourceMsg;
use midi_nostd::sound_source_msgs::SoundSourceMsgs;
use midi_nostd::sound_source_msgs::SoundSourceOscillatorInit;
use midi_nostd::sound_source_msgs::SoundSourceValue;
use midi_nostd::sound_sources::SoundSources;
use midly::Smf;

use portaudio as pa;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 24_000.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 200;

fn midly_exploration() {
    let smf = Smf::parse(include_bytes!("../assets/twinkle.mid")).unwrap();

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

fn run() -> Result<(), pa::Error> {
    println!(
        "Midi STD Test.  SR = {}, BufSize = {}",
        SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    let mut all_pools = midi_nostd::sound_sources_impl::SoundSourcesImpl::<
        midi_nostd::sound_sample::SoundSampleI32,
        24000,
        32,
        32,
        32,
    >::default();
    let wave_id = midi_nostd::oscillator::create_oscillator(
        &mut all_pools,
        SoundSourceOscillatorInit::new(
            midi_nostd::sound_source_msgs::OscillatorType::PulseWidth,
            30000,
            50,
            50,
        ),
    );

    let adsr_id = create_adsr(
        &mut all_pools,
        SoundSourceAdsrInit::new(
            SoundScale::new_percent(100),
            SoundScale::new_percent(100),
            6 * 24,
            33 * 24,
            30 * 24,
        ),
    );

    let amp_id = create_amp_mixer(
        &mut all_pools,
        SoundSourceAmpMixerInit::new(wave_id, adsr_id),
    );

    // Initialise sinusoidal wavetable.
    let mut left_phase = 0;
    let mut right_phase = 0;

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::StreamFlags::CLIP_OFF;
    let mut sent_count: u32 = 0;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        let mut new_msgs = midi_nostd::sound_source_msgs::SoundSourceMsgs::default();
        for _ in 0..frames {
            all_pools.update(&mut new_msgs);
            let current = all_pools.get_next(&amp_id);
            let converted: f32 = (((current.to_u16() as i32) - 32768) as f32) / 32768.0;
            buffer[idx] = converted; //sine[left_phase];
            buffer[idx + 1] = converted; // = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 3;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            idx += 2;
            sent_count = sent_count + 1;
            if sent_count == 3000 {
                let mut msgs = SoundSourceMsgs::default();
                msgs.append(SoundSourceMsg::new(
                    adsr_id.clone(),
                    all_pools.get_top_id(),
                    SoundSourceKey::ReleaseAdsr,
                    SoundSourceValue::default(),
                ));
                all_pools.process_and_clear_msgs(&mut msgs);
            }
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
