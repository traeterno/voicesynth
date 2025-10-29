use std::f32::consts::PI;

use random::Source;

#[derive(Clone, Copy)]
pub enum WaveType
{
	Sine(f32),
	Saw,
	Square(f32),
	Noise
}

impl ToString for WaveType
{
	fn to_string(&self) -> String
	{
		match *self
		{
			Self::Sine(phase) => format!("Sine ({phase}deg)"),
			Self::Saw => format!("Sawtooth"),
			Self::Square(pwm) => format!("Square ({}%)", (pwm * 100.0).round()),
			Self::Noise => format!("Noise")
		}
	}
}

#[derive(Clone, Copy)]
pub enum Filter
{
	None,
	LowPass(f32),
	HighPass(f32),
	BandPass(f32),
	Notch(f32)
}

#[derive(Clone, Copy)]
pub enum Envelope
{
	None(f32),
	AttackDecay(f32, f32),
	Sine(f32, f32)
}

pub struct Generator
{
	wave: WaveType,
	freq: f32,
	tick: u32,
	channel: bool,
	env: Envelope,
	volume: f32,
	filter: Filter,
	rand: random::Xorshift128Plus,
}

impl Generator
{
	pub fn new(
		wave: WaveType, freq: f32,
		env: Envelope, volume: f32, filter: Filter
	) -> Self
	{
		Self
		{
			wave, freq, tick: 0, channel: false,
			env, volume, filter,
			rand: random::default(freq as u64)
		}
	}

	pub fn get(&mut self, sampleRate: u32) -> f32
	{
		let t = self.tick as f32 / sampleRate as f32;
		let wave = match self.wave
		{
			WaveType::Sine(phase) =>
			{
				(2.0 * PI * t * self.freq + phase / (360.0 * self.freq)).sin()
			}
			WaveType::Saw => 2.0 * (t * self.freq - (0.5 + t * self.freq).floor()),
			WaveType::Square(pwm) => if (t * self.freq).fract() > pwm { 1.0 } else { 0.0 },
			WaveType::Noise => self.rand.read::<i16>() as f32 / i16::MAX as f32
		};

		let wave = match self.filter
		{
			Filter::None => wave,
			Filter::LowPass(a) => if self.freq <= a { 1.0 } else { 1.0 + a - self.freq },
			_ => wave
		};

		self.channel = !self.channel;
		if !self.channel { self.tick += 1; }
		wave * self.calcEnvelope(self.tick, sampleRate) * self.volume
	}

	fn calcEnvelope(&self, tick: u32, sampleRate: u32) -> f32
	{
		let t = tick as f32 / sampleRate as f32;
		match self.env
		{
			Envelope::None(_) => 1.0,
			Envelope::AttackDecay(attack, decay) =>
			{
				if attack > 0.0
				{
					if t < attack { return t / attack }
					else if decay <= 0.0 { return 1.0 - (t - attack) }
					else { return 1.0 - (t - attack) / decay }
				}
				else { return 1.0 - t / decay }
			}
			Envelope::Sine(freq, duration) =>
			{
				((2.0 * PI * t * freq).sin() + 1.0) / 2.0 * (1.0 - t / duration)
			}
		}
	}

	pub fn isActive(&self, sampleRate: u32) -> bool
	{
		let t = self.tick as f32 / sampleRate as f32;
		match self.env
		{
			Envelope::None(duration) => t <= duration,
			Envelope::AttackDecay(attack, decay) =>
			{
				if attack <= 0.0 { return t <= decay }
				else { return t <= attack + decay }
			}
			Envelope::Sine(_, duration) => t <= duration
		}
	}
}