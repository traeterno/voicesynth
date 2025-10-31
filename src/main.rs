#![allow(static_mut_refs, non_snake_case, dead_code, unused_imports)]

mod aesound;

use std::sync::LazyLock;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use sdl2::keyboard::{Keycode, Mod};

use crate::aesound::{AudioFile::AudioFile, Generator::{Envelope, Filter, Generator, WaveType}, SoundSource::SoundSource};

const NOTES: [f32; 12] = [
	16.35, // 00 - C
	17.32, // 01 - C# | Db
	18.35, // 02 - D
	19.45, // 03 - D# | Eb
	20.60, // 04 - E
	21.83, // 05 - F
	23.12, // 06 - F# | Gb
	24.50, // 07 - G
	25.96, // 08 - G# | Ab
	27.50, // 09 - A
	29.14, // 10 - A# | Bb
	30.87, // 11 - B
];

static mut SOUNDS: LazyLock<Vec<SoundSource>> = LazyLock::new(|| vec![]);

fn add(octave: i32, note: i32, waveform: WaveType, harmonics: u32, env: Envelope)
{
	let freq = NOTES[note as usize];
	let f0 = 2.0_f32.powi(octave);
	
	for i in 1..=harmonics
	{
		unsafe
		{
			(*SOUNDS).push(SoundSource::ProcGen(Generator::new(
				waveform, freq * f0 * i as f32,
				env,
				1.0 / i as f32, Filter::None
			)));
		}
	}
}

fn main()
{
	let host = cpal::default_host();
	let device = host.default_output_device().unwrap();
	println!("Output device: {}", device.name().unwrap());
	let config =
		device.supported_output_configs().unwrap()
		.filter(|x|
			{ x.sample_format() == cpal::SampleFormat::F32 }
		).nth(0).unwrap().with_max_sample_rate();
	let rate = config.sample_rate().0;
	
	let stream = device.build_output_stream(
		&config.config(),
		move |data: &mut [f32], _: &cpal::OutputCallbackInfo| unsafe
		{
			for x in data
			{
				*x = 0.0;
				for mut i in 0..(*SOUNDS).len() as isize
				{
					if i >= (*SOUNDS).len() as isize { break; }
					let s = &mut (*SOUNDS)[i as usize];
					if !s.isActive(rate)
					{
						i -= 1;
						(*SOUNDS).remove((i + 1) as usize);
						continue;
					}
					*x += s.get(rate);
				}
			}
		},
		move |err| { println!("{err:?}"); },
		None
	).unwrap();
	let _ = stream.play();

	let ctx = sdl2::init().unwrap();
	let video = ctx.video().unwrap();
	let win = video.window("synth", 600, 200)
		.build().unwrap();
	let mut canvas = win.into_canvas().accelerated().build().unwrap();

	let mut ep = ctx.event_pump().unwrap();

	let ttf = sdl2::ttf::init().unwrap();
	let f = ttf.load_font(
		"C:/Windows/Fonts/arial.ttf", 24
	).unwrap();
	let tc = canvas.texture_creator();

	let mut octave = 1;
	let mut semitone = 0;
	let mut lpf = 1000.0;
	let mut harmonics = 1;
	let mut attack = 0.0;
	let mut decay = 1.0;
	let mut pwm = 0.5;
	let mut wave = WaveType::Triangle;

	// unsafe
	// {
	// 	if let Some(x) = AudioFile::new(String::from("./test.ogg"))
	// 	{
	// 		(*SOUNDS).push(SoundSource::AudioFile(x));
	// 	}
	// }

	'running: loop
	{
		for e in ep.poll_iter()
		{
			match e
			{
				sdl2::event::Event::Quit { .. } => { break 'running }
				sdl2::event::Event::KeyDown { keycode, keymod, .. } =>
				{
					if let Some(k) = keycode
					{
						let mut note = None;
						if k == Keycode::Escape { break 'running; }
						if k == Keycode::H { octave = 1; }
						if k == Keycode::J { octave = 2; }
						if k == Keycode::K { octave = 3; }
						if k == Keycode::L { octave = 4; }
						if k == Keycode::Num7 { semitone = -1; }
						if k == Keycode::Num8 { semitone =  0; }
						if k == Keycode::Num9 { semitone =  1; }
						if k == Keycode::Q { note = Some(0 + semitone.max(0)); } // C
						if k == Keycode::W { note = Some(2 + semitone); } // D
						if k == Keycode::E { note = Some(4 + semitone.min(0)); } // E
						if k == Keycode::R { note = Some(5 + semitone.max(0)); } // F
						if k == Keycode::T { note = Some(7 + semitone); } // G
						if k == Keycode::Y { note = Some(9 + semitone); } // A
						if k == Keycode::U { note = Some(11 + semitone.min(0)); } // B
						if k == Keycode::Z { wave = WaveType::Sine; }
						if k == Keycode::X { wave = WaveType::Saw; }
						if k == Keycode::C { wave = WaveType::Square(pwm); }
						if k == Keycode::V { wave = WaveType::SineOverdrive(pwm); }
						if k == Keycode::B { wave = WaveType::Triangle; }
						if k == Keycode::Up { lpf += 50.0; }
						if k == Keycode::Down { lpf -= 50.0; }
						if k == Keycode::Left { harmonics = (harmonics - 1).max(1); }
						if k == Keycode::Right { harmonics += 1; }
						if k == Keycode::A
						{
							if keymod.intersects(Mod::LSHIFTMOD) { attack -= 0.1; }
							else { attack += 0.1; }
						}
						if k == Keycode::D
						{
							if keymod.intersects(Mod::LSHIFTMOD) { decay -= 0.1; }
							else { decay += 0.1; }
						}
						if k == Keycode::S
						{
							if keymod.intersects(Mod::LSHIFTMOD) { pwm -= 0.05; }
							else { pwm += 0.05; }
							wave = match wave
							{
								WaveType::SineOverdrive(_) => WaveType::SineOverdrive(pwm),
								_ => WaveType::Square(pwm)
							}
						}
						if let Some(n) = note
						{
							add(octave, n, wave, harmonics,
								Envelope::AttackDecay(attack, decay)
							);
						}
					}
				}
				_ => {}
			}
		}
		canvas.clear();
		unsafe
		{
			let surface = f.render(
				&format!("Octave: {octave} / Semitone: {semitone}")
			).solid(sdl2::pixels::Color::WHITE).unwrap();
			let (w, h) = surface.size();
			let _ = canvas.copy(
				&tc.create_texture_from_surface(surface).unwrap(), None,
				Some(sdl2::rect::Rect::new(0, 0, w, h))
			);
			let y = h;
			let surface = f.render(
				&format!("Gens count: {} / LPF: {lpf}", (*SOUNDS).len())
			).solid(sdl2::pixels::Color::WHITE).unwrap();
			let (w, h) = surface.size();
			let _ = canvas.copy(
				&tc.create_texture_from_surface(surface).unwrap(), None,
				Some(sdl2::rect::Rect::new(0, y as i32, w, h))
			);
			let y = y + h;
			let surface = f.render(
				&format!("Waveform: {} / Harmonics: {harmonics}", wave.to_string())
			).solid(sdl2::pixels::Color::WHITE).unwrap();
			let (w, h) = surface.size();
			let _ = canvas.copy(
				&tc.create_texture_from_surface(surface).unwrap(), None,
				Some(sdl2::rect::Rect::new(0, y as i32, w, h))
			);
			let y = y + h;
			let surface = f.render(
				&format!("Attack: {} / Decay: {}",
					(attack * 10.0 + 0.5).floor() as f32 / 10.0,
					(decay * 100.0 + 0.5).floor() as f32 / 100.0
				)
			).solid(sdl2::pixels::Color::WHITE).unwrap();
			let (w, h) = surface.size();
			let _ = canvas.copy(
				&tc.create_texture_from_surface(surface).unwrap(), None,
				Some(sdl2::rect::Rect::new(0, y as i32, w, h))
			);
		}
		canvas.present();
	}
}