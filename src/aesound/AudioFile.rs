use std::{fs::File, time::Instant};

use lewton::inside_ogg::OggStreamReader;

pub struct AudioFile
{
	ogg: OggStreamReader<std::fs::File>,
	buffer: Vec<Vec<i16>>,
	currentPacket: usize,
	currentSample: usize
}

impl AudioFile
{
	pub fn new(path: String) -> Option<Self>
	{
		if let Ok(f) = File::open(&path)
		{
			if let Ok(ogg) = OggStreamReader::new(f)
			{
				if ogg.ident_hdr.audio_channels > 2
				{
					println!("AudioFile {path}: can't use more than 2 audio channels.");
					return None;
				}
				return Some(Self
				{
					ogg, buffer: vec![],
					currentPacket: 0, currentSample: 0
				})
			}
		}
		None
	}

	pub fn get(&mut self) -> f32
	{
		while self.currentPacket == self.buffer.len()
		{
			if let Ok(Some(x)) = self.ogg.read_dec_packet_itl()
			{
				if x.len() > 0 { self.buffer.push(x); }
			}
		}
		let p = &self.buffer[self.currentPacket];
		let s = p[self.currentSample];
		self.currentSample += 1;
		if self.currentSample >= p.len() { self.currentPacket += 1; self.currentSample = 0; }
		return s as f32 / i16::MAX as f32;
	}

	pub fn isActive(&self) -> bool
	{
		true
	}
}