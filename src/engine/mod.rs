//! Portable synthesis. Original x86 pointers and Delphi ABI must not leak here.
//! Transition assembly and Anatol's period-based speed algorithm.

#[derive(Debug)]
pub struct PcmAudio {
    pub sample_rate: u32,
    pub channels: u16,
    /// Unsigned 8-bit PCM, matching the original WAV template.
    pub samples: Vec<u8>,
}

pub const MAX_AUDIO_BYTES: usize = 64 * 1024 * 1024;

pub fn synthesize(
    phonemes: &[u8],
    voice: &crate::resources::VoiceData,
    speed: u8,
) -> std::io::Result<PcmAudio> {
    let mut samples = Vec::new();
    for pair in phonemes.windows(2) {
        let second = if voice.voice == 2
            && matches!(
                pair[0] - 0xd2,
                0 | 1 | 2 | 3 | 4 | 5 | 14 | 19 | 22 | 28 | 33 | 39
            )
            && matches!(pair[1] - 0xd2, 12 | 13 | 22 | 27 | 30 | 34 | 35)
        {
            0xfa
        } else {
            pair[1]
        };
        let Some(encoded) = voice.transition(pair[0], second) else {
            continue;
        };
        let block = if voice.voice == 1 {
            anatol_sample(encoded, pair[0], second, speed)?
        } else {
            encoded.to_vec()
        };
        if samples.len().saturating_add(block.len()) > MAX_AUDIO_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "audio exceeds 64 MiB size limit",
            ));
        }
        if samples.is_empty() {
            samples.extend_from_slice(&block);
        } else {
            if samples.len() < 17 || block.len() < 17 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "sample is too short for crossfade",
                ));
            }
            let start = samples.len() - 17;
            for (index, &next) in block.iter().take(17).enumerate() {
                let weight = index as f64 / 16.0;
                let previous = samples[start + index];
                let left = ((1.0 - weight) * (f64::from(previous) - 128.0) + 128.0)
                    .round_ties_even() as i64;
                let right = (weight * (f64::from(next) - 128.0)).round_ties_even() as i64;
                samples[start + index] = (left + right) as u8;
            }
            samples.extend_from_slice(&block[16..block.len() - 1]);
        }
    }
    if samples.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "text produces no speech",
        ));
    }
    Ok(PcmAudio {
        sample_rate: 11025,
        channels: 1,
        samples,
    })
}

fn anatol_sample(encoded: &[u8], first: u8, second: u8, speed: u8) -> std::io::Result<Vec<u8>> {
    use std::io::{Error, ErrorKind};
    let invalid = || Error::new(ErrorKind::InvalidData, "invalid Anatol sample descriptor");
    let header = usize::from(*encoded.first().ok_or_else(invalid)?);
    if speed == 5 {
        return encoded
            .get(header..)
            .map(<[u8]>::to_vec)
            .ok_or_else(invalid);
    }
    let divisor = usize::from(if speed < 5 {
        speed
    } else if first - 0xd2 < 6 || second - 0xd2 < 6 {
        11 - speed
    } else {
        12 - speed
    });
    let count = usize::from(*encoded.get(1).ok_or_else(invalid)?);
    let mut output = Vec::new();
    for record in 0..count {
        let offset = usize::from(*encoded.get(2 + record).ok_or_else(invalid)?);
        let desc = encoded.get(offset..offset + 5).ok_or_else(invalid)?;
        let source = usize::from(u16::from_le_bytes([desc[1], desc[2]]));
        let length = usize::from(u16::from_le_bytes([desc[3], desc[4]]));
        let data = encoded.get(source..source + length).ok_or_else(invalid)?;
        match desc[0] {
            1 => output.resize(
                output.len()
                    + if speed > 5 {
                        length - length / divisor
                    } else {
                        length + length / divisor
                    },
                128,
            ),
            2 => {
                let step = if first == 0xf2 { 35 } else { 70 };
                for (index, part) in data.chunks_exact(step).enumerate() {
                    if (index + 1) % divisor != 0 || speed < 5 {
                        output.extend_from_slice(part);
                    }
                    if (index + 1) % divisor == 0 && speed < 5 {
                        output.extend_from_slice(part);
                    }
                }
            }
            3 => {
                let count = usize::from(*encoded.get(offset + 5).ok_or_else(invalid)?);
                let periods = encoded
                    .get(offset + 6..offset + 6 + count)
                    .ok_or_else(invalid)?;
                let mut consumed = 0;
                if record == 0 {
                    output.extend_from_slice(data.get(..8).ok_or_else(invalid)?);
                    consumed = 8;
                }
                for (index, &period) in periods.iter().enumerate() {
                    let end = consumed + usize::from(period);
                    let part = data.get(consumed..end).ok_or_else(invalid)?;
                    if (index + 1) % divisor != 0 || speed < 5 {
                        output.extend_from_slice(part);
                    }
                    if (index + 1) % divisor == 0 && speed < 5 {
                        output.extend_from_slice(part);
                    }
                    consumed = end;
                }
            }
            _ => return Err(invalid()),
        }
    }
    Ok(output)
}
