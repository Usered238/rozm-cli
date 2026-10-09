//! Read-only loading of validated BSNbn, skfs and snf binary formats.
//! Lookup: ROZM_DATA_DIR, otherwise data/ beside the executable.

use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug)]
pub struct ResourceLocation {
    pub root: PathBuf,
}

pub struct VoiceData {
    pub(crate) voice: u8,
    pub(crate) offsets: Vec<i32>,
    pub(crate) lengths: Vec<i32>,
    pub(crate) samples: Vec<u8>,
}

const ALPHABET: &[u8] = &[
    0x21, 0xe0, 0xe1, 0xe2, 0xe3, 0xb4, 0xe4, 0xe5, 0xba, 0xe6, 0xe7, 0xe8, 0xb3, 0xbf, 0xe9, 0xea,
    0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfc,
    0xfe, 0xff, 0x27, 0x5c, 0x2d, 0x20, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
    0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x23, 0x24,
];

pub struct PronunciationDictionary {
    encoded: Vec<u8>,
    records: Vec<(usize, usize)>,
    corrections: HashMap<Vec<u8>, u8>,
}

impl PronunciationDictionary {
    pub fn stress(&self, word: &[u8]) -> u8 {
        if let Some(&stress) = self.corrections.get(word) {
            return stress;
        }
        let record = self.records.binary_search_by(|&(start, end)| {
            self.encoded[start..end]
                .iter()
                .map(|&code| {
                    if code < 65 {
                        ALPHABET[code as usize]
                    } else {
                        code
                    }
                })
                .cmp(word.iter().copied())
        });
        match record {
            Ok(index) => {
                let code = self.encoded[self.records[index].1];
                if code < 65 {
                    ALPHABET.get(code as usize).copied().unwrap_or(b'8')
                } else {
                    code
                }
            }
            Err(_) => b'8',
        }
    }
}

impl ResourceLocation {
    pub fn load_dictionary(&self) -> io::Result<PronunciationDictionary> {
        let read = |name: &str| -> io::Result<Vec<u8>> {
            let path = self.root.join(name);
            let size = fs::metadata(&path)
                .map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        format!("resource {}: {error}", path.display()),
                    )
                })?
                .len();
            if size > 32 * 1024 * 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "resource exceeds size limit",
                ));
            }
            fs::read(&path)
        };
        let encoded = read("BSNbn")?;
        let mut records = Vec::new();
        let mut start = 0;
        for (end, &code) in encoded.iter().enumerate() {
            if (38..81).contains(&code) {
                records.push((start, end));
                start = end + 1;
            }
        }
        // Original dictionary ends with a standalone '!' sentinel and has a few
        // literal (uncompressed) character bytes within otherwise encoded words.
        if records.is_empty() || encoded.get(start..) != Some(&[0x21][..]) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid BSNbn record boundaries",
            ));
        }
        let skf = read("skfs/skf")?;
        let wif = read("skfs/wif")?;
        let wlf = read("skfs/wlf")?;
        if wif.len() < 4 || wif.len() % 4 != 0 || wlf.len() != wif.len() / 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid correction dictionary tables",
            ));
        }
        let count = u32::from_le_bytes(wif[..4].try_into().unwrap()) as usize;
        if count.checked_add(1) != Some(wlf.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid correction dictionary count",
            ));
        }
        let mut corrections = HashMap::new();
        for index in 1..=count {
            let offset =
                u32::from_le_bytes(wif[index * 4..index * 4 + 4].try_into().unwrap()) as usize;
            let length = usize::from(wlf[index]);
            let record = skf
                .get(
                    offset..offset.checked_add(length).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "correction offset overflow")
                    })?,
                )
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid correction record")
                })?;
            corrections.insert(
                record[..record.len() - 1].to_vec(),
                record[record.len() - 1],
            );
        }
        Ok(PronunciationDictionary {
            encoded,
            records,
            corrections,
        })
    }
    pub fn discover() -> io::Result<Self> {
        let root = match std::env::var_os("ROZM_DATA_DIR") {
            Some(path) => PathBuf::from(path),
            None => std::env::current_exe()?
                .parent()
                .ok_or_else(|| io::Error::other("executable directory is unavailable"))?
                .join("data"),
        };
        Ok(Self { root })
    }

    pub fn load_voice(&self, voice: u8) -> io::Result<VoiceData> {
        let root = self.root.join(format!("snf{voice}"));
        let read = |name: &str| -> io::Result<Vec<u8>> {
            let path = root.join(name);
            let size = fs::metadata(&path)
                .map_err(|error| {
                    io::Error::new(
                        error.kind(),
                        format!("resource {}: {error}", path.display()),
                    )
                })?
                .len();
            if size > 32 * 1024 * 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "voice resource exceeds size limit",
                ));
            }
            fs::read(&path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("resource {}: {error}", path.display()),
                )
            })
        };
        let table = |name: &str| -> io::Result<Vec<i32>> {
            let bytes = read(name)?;
            if bytes.len() != 42 * 42 * 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid {name}: expected 7056 bytes"),
                ));
            }
            Ok(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|bytes| i32::from_le_bytes(*bytes))
                .collect())
        };
        let mut data = VoiceData {
            voice,
            offsets: table("ip2f")?,
            lengths: table("lp2f")?,
            samples: read("Sd2f")?,
        };
        // Original Sd2f ends one byte before the final silence transition. Delphi
        // reads it into zero-initialized storage; reproduce that trailing zero.
        let maximum_end = data
            .offsets
            .iter()
            .zip(&data.lengths)
            .filter(|(offset, length)| **offset >= 0 && **length > 0)
            .map(|(&offset, &length)| offset as usize + length as usize)
            .max()
            .unwrap_or(0);
        if maximum_end == data.samples.len() + 1 {
            data.samples.push(0);
        }
        for (&offset, &length) in data.offsets.iter().zip(&data.lengths) {
            if length > 0 && (offset < 0 || offset as usize + length as usize > data.samples.len())
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "voice sample range is outside Sd2f",
                ));
            }
        }
        Ok(data)
    }
}

impl VoiceData {
    pub(crate) fn transition(&self, first: u8, second: u8) -> Option<&[u8]> {
        let index =
            usize::from(first.checked_sub(0xd2)?) * 42 + usize::from(second.checked_sub(0xd2)?);
        let (&offset, &length) = (self.offsets.get(index)?, self.lengths.get(index)?);
        if length <= 0 {
            return None;
        }
        self.samples
            .get(offset as usize..offset as usize + length as usize)
    }
}
