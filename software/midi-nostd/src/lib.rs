#![cfg_attr(not(test), no_std)]

pub mod adsr;
pub mod amp_adder;
pub mod amp_mixer;
mod free_list;
pub mod midi;
pub mod midi_notes;
pub mod note;
pub mod oscillator;
pub mod sound_sample;
pub mod sound_source;
pub mod sound_source_core;
pub mod sound_source_id;
mod sound_source_pool;
pub mod sound_source_pool_impl;
pub mod sound_sources;
pub mod sound_sources_impl;
mod wave_tables;
