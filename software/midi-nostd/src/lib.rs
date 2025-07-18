#![cfg_attr(not(test), no_std)]

pub mod adsr;
pub mod amp_mixer;
mod free_list;
pub mod midi;
pub mod midi_notes;
pub mod oscillator;
pub mod sound_sample;
pub mod sound_source;
pub mod sound_source_id;
pub mod sound_source_msgs;
mod sound_source_pool;
pub mod sound_source_pool_impl;
pub mod sound_sources;
pub mod sound_sources_impl;
pub mod top;
mod wave_tables;
