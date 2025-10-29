use crate::aesound::{AudioFile::AudioFile, Generator::Generator};

pub enum SoundSource
{
	ProcGen(Generator),
	AudioFile(AudioFile)
}

impl SoundSource
{
	pub fn get(&mut self, sampleRate: u32) -> f32
	{
		match self
		{
			Self::ProcGen(g) => g.get(sampleRate),
			Self::AudioFile(f) => f.get()
		}
	}

	pub fn isActive(&self, sampleRate: u32) -> bool
	{
		match self
		{
			Self::ProcGen(g) => g.isActive(sampleRate),
			Self::AudioFile(f) => f.isActive()
		}
	}
}