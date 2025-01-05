use crate::sound_sample::SoundSample;

pub trait SoundSource<T: SoundSample> {
    fn get_next(self: &mut Self) -> T;
}
