//! Validated Unicode input, original text transformations, stress and phonetics.

use crate::resources::PronunciationDictionary;
use std::io;
mod latin;
mod numbers;
mod symbols;

fn vowel(byte: u8) -> bool {
    matches!(
        byte,
        0xaa | 0xaf
            | 0xb2
            | 0xb3
            | 0xba
            | 0xbf
            | 0xc0
            | 0xc5
            | 0xc8
            | 0xce
            | 0xd3
            | 0xde
            | 0xdf
            | 0xe0
            | 0xe5
            | 0xe8
            | 0xee
            | 0xf3
            | 0xfe
            | 0xff
    )
}
fn stressed(byte: u8) -> u8 {
    match byte {
        0xb3 => 0xb2,
        0xbf => 0xaf,
        0xba => 0xaa,
        0xe0 | 0xe5 | 0xe8 | 0xee | 0xf3 | 0xfe | 0xff => byte - 32,
        _ => byte,
    }
}
fn punctuation(byte: u8) -> bool {
    matches!(
        byte,
        b'!' | b'(' | b')' | b',' | b'.' | b':' | b';' | b'?' | 13 | 0x85
    )
}

pub fn encode(text: &str) -> io::Result<Vec<u8>> {
    let mut encoded = Vec::new();
    for ch in text.chars() {
        let byte = match ch {
            '№' => 0xb9,
            '\u{2019}' | '\u{02bc}' => b'\'',
            '\u{201c}' | '\u{201d}' | '«' | '»' => b'"',
            '\u{2013}' | '\u{2014}' => b'-',
            '\u{2026}' => 0x85,
            'Ґ' => 0xa5,
            'ґ' => 0xb4,
            'Є' => 0xaa,
            'є' => 0xba,
            'І' => 0xb2,
            'і' => 0xb3,
            'Ї' => 0xaf,
            'ї' => 0xbf,
            '\t' => b' ',
            ch if ch.is_ascii() && ch != '\0' => ch as u8,
            'А'..='я' => 0xc0 + (ch as u32 - 'А' as u32) as u8,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unsupported text character U+{:04X}", ch as u32),
                ));
            }
        };
        encoded.push(byte);
    }
    Ok(encoded)
}

pub fn validate(text: &str) -> io::Result<()> {
    if text.len() > 32768 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "text exceeds 32768 UTF-8 bytes",
        ));
    }
    if !text.chars().any(|ch| ch.is_alphanumeric()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "text must contain a word or number",
        ));
    }
    encode(text)?;
    Ok(())
}

fn normalize(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    if bytes
        .first()
        .is_some_and(|b| *b == b' ' || b.is_ascii_digit())
    {
        out.push(b'.');
    }
    for (i, &b) in bytes.iter().enumerate() {
        if (0xe0..0xfa).contains(&b)
            || matches!(b, 0xfc | 0xfe | 0xff | b'\'' | b'\\')
            || punctuation(b)
        {
            out.push(b);
        } else if b == b'`' {
            out.push(b'\'');
        } else if b == b' ' && bytes.get(i + 1) == Some(&b'-') {
            out.push(b',');
        } else if b == b' ' && bytes.get(i + 1).is_some_and(|b| punctuation(*b)) {
        } else if (0xc0..0xda).contains(&b) || matches!(b, 0xdf | 0xde | 0xdc) {
            out.push(b + 32);
        } else if matches!(b, 0xb2 | 0xb3 | b'I' | b'i') {
            out.push(0xb3);
        } else if matches!(b, 0xaf | 0xbf) {
            out.push(0xbf);
        } else if matches!(b, 0xaa | 0xba) {
            out.push(0xba);
        } else if out.last().is_some_and(|b| *b != b' ') {
            out.push(b' ');
        }
    }
    out
}

fn mark_word(mut word: Vec<u8>, previous: &[u8], dictionary: &PronunciationDictionary) -> Vec<u8> {
    if word
        .iter()
        .enumerate()
        .any(|(i, &b)| vowel(b) && word.get(i + 1).is_some_and(|b| matches!(b, b'\\' | b'\'')))
    {
        let mut out = Vec::new();
        let mut i = 0;
        while i < word.len() {
            let b = word[i];
            if vowel(b) && word.get(i + 1).is_some_and(|b| matches!(b, b'\\' | b'\'')) {
                out.push(stressed(b));
                i += 2;
            } else {
                out.push(b);
                i += 1;
            }
        }
        return out;
    }
    let mut marker = dictionary.stress(&word);
    if marker == b'8' && word.len() > 2 {
        let mut stem = word.clone();
        if stem.ends_with(&[0xf1, 0xfc]) {
            let len = stem.len();
            stem[len - 1] = 0xff;
        }
        if [
            [0xe0, 0xff],
            [0xf3, 0xfe],
            [0xe5, 0xba],
            [0xb3, 0xbf],
            [0xff, 0xff],
            [0xba, 0xba],
        ]
        .iter()
        .any(|ending| stem.ends_with(ending))
        {
            stem.pop();
        }
        marker = dictionary.stress(&stem);
    }
    let mut ordinal = if marker > b'9' {
        marker - b'9'
    } else {
        marker.saturating_sub(b'0')
    };
    if (b'B'..b'P').contains(&marker) {
        let value = marker - 60;
        let alternative = if value.is_multiple_of(4) {
            4
        } else {
            value % 4
        };
        ordinal = (value - alternative) / 4;
        let prepositions = [
            "до",
            "від",
            "од",
            "з",
            "із",
            "біля",
            "коло",
            "для",
            "без",
            "крім",
            "нема",
            "немає",
            "окрім",
            "поблизу",
        ];
        if prepositions
            .iter()
            .any(|w| encode(w).is_ok_and(|w| w == previous))
            || [
                &[0xee, 0xbf][..],
                &[0xba, 0xbf][..],
                &[0xee, 0xe3, 0xee][..],
            ]
            .iter()
            .any(|s| previous.ends_with(s))
        {
            ordinal = alternative;
        }
    }
    if ["мене", "тебе", "себе"]
        .iter()
        .any(|w| encode(w).is_ok_and(|w| w == word))
        && [
            "у",
            "в",
            "до",
            "на",
            "по",
            "від",
            "од",
            "про",
            "для",
            "за",
            "під",
            "із",
            "з",
            "біля",
            "крім",
            "без",
            "повз",
            "окрім",
            "коло",
        ]
        .iter()
        .any(|w| encode(w).is_ok_and(|w| w == previous))
    {
        ordinal = 1;
        marker = b'1';
    }
    let mut count = 0;
    let length = word.len();
    for b in &mut word {
        if vowel(*b) {
            count += 1;
            if (marker == b'8' && length > 1) || (marker != b'8' && count == ordinal) {
                *b = stressed(*b);
            }
        }
    }
    word
}

fn phonetic(bytes: &[u8]) -> Vec<u8> {
    let mut out = if bytes.first() == Some(&b'.') {
        vec![0xfa]
    } else {
        Vec::new()
    };
    let mut i = 0;
    let soft = |b| match b {
        0xe4 => Some(0xd8),
        0xe7 => Some(0xd9),
        0xeb => Some(0xda),
        0xed => Some(0xdb),
        0xf0 => Some(0xdc),
        0xf1 => Some(0xdd),
        0xf2 => Some(0xde),
        0xf6 => Some(0xdf),
        _ => None,
    };
    let vowel_code = |b| match b {
        0xaa => 0xd3,
        0xba => 0xe5,
        0xaf => 0xd5,
        0xbf => 0xf9,
        0xb2 => 0xd5,
        0xb3 => 0xf9,
        0xde => 0xd7,
        0xfe => 0xf3,
        0xdf => 0xd2,
        0xff => 0xe0,
        _ => b,
    };
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied().unwrap_or(0);
        if let Some(s) = soft(b) {
            if next == 0xfc {
                out.push(s);
                i += 2;
                continue;
            }
            if matches!(next, 0xaa | 0xb2 | 0xb3 | 0xba | 0xde | 0xdf | 0xfe | 0xff) {
                out.extend([s, vowel_code(next)]);
                i += 2;
                continue;
            }
        }
        if b == b' '
            && bytes
                .get(i.wrapping_sub(1))
                .is_some_and(|b| !punctuation(*b))
            && vowel(next)
        {
            out.push(0xfa);
            i += 1;
            continue;
        }
        if b == b'\'' {
            i += 1;
            continue;
        }
        let replacement = if bytes[i..].starts_with(&[0xf7, 0xf7, 0xff]) {
            Some(&[0xf7, 0xf9, 0xe0][..])
        } else if bytes[i..].starts_with(&[0xf7, 0xf7, 0xfe]) {
            Some(&[0xf7, 0xf9, 0xf3][..])
        } else if bytes[i..].starts_with(&[0xf7, 0xf7, 0xb3]) {
            Some(&[0xf7, 0xf9][..])
        } else if bytes[i..].starts_with(&[0xe6, 0xe6, 0xff]) {
            Some(&[0xe6, 0xf9, 0xe0][..])
        } else if bytes[i..].starts_with(&[0xe6, 0xe6, 0xb3]) {
            Some(&[0xe6, 0xf9][..])
        } else {
            None
        };
        if let Some(r) = replacement {
            out.extend_from_slice(r);
            i += 3;
            continue;
        }
        if matches!(b, 0xaa | 0xaf | 0xba | 0xbf | 0xde | 0xdf | 0xfe | 0xff) {
            out.extend([0xe9, vowel_code(b)]);
        } else if (0xe0..0xf9).contains(&b) {
            out.push(b);
        } else if matches!(b, 0xc0 | 0xc5 | 0xc8 | 0xce | 0xd3 | 0xb2 | 0xb3) {
            out.push(match b {
                0xc0 => 0xd2,
                0xc5 => 0xd3,
                0xc8 => 0xd4,
                0xce => 0xd6,
                0xd3 => 0xd7,
                0xb2 => 0xd5,
                _ => 0xf9,
            });
        } else if b == 0xf9 {
            out.extend([0xf8, 0xf7]);
        } else if punctuation(b) {
            out.push(0xfa);
        }
        i += 1;
    }
    // Original inserts silence between selected adjacent consonants.
    let consonant = |b| {
        matches!(
            b,
            0xea | 0xef
                | 0xf2
                | 0xf6
                | 0xf7
                | 0xde
                | 0xdf
                | 0xd8
                | 0xd9
                | 0xda
                | 0xdb
                | 0xdc
                | 0xe1
                | 0xe2
                | 0xe3
                | 0xe4
                | 0xe6
                | 0xe7
                | 0xe9
                | 0xeb
                | 0xec
                | 0xed
                | 0xf0
                | 0xdd
                | 0xf1
                | 0xf4
                | 0xf5
                | 0xf8
        )
    };
    let mut result = Vec::new();
    for (i, &b) in out.iter().enumerate() {
        result.push(b);
        if consonant(b) && out.get(i + 1).is_some_and(|b| consonant(*b)) {
            result.push(0xfa);
        }
    }
    result.push(0xfa);
    if result.first() != Some(&0xfa) {
        result.insert(0, 0xfa);
    }
    result
}

pub fn phonemes(text: &str, dictionary: &PronunciationDictionary) -> io::Result<Vec<u8>> {
    let bytes = normalize(&encode(&latin::expand(&numbers::expand(
        &symbols::expand(text),
    )))?);
    let mut marked = Vec::new();
    let mut previous = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b' ' || b == b'-' || punctuation(b) {
            marked.push(b);
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'-' && !punctuation(bytes[i]) {
            i += 1;
        }
        let word = bytes[start..i].to_vec();
        marked.extend(mark_word(word.clone(), &previous, dictionary));
        marked.push(b' ');
        previous = word;
    }
    Ok(phonetic(&marked))
}

pub struct Clause {
    pub text: String,
    pub voice: u8,
}

pub fn clauses(text: &str, mut voice: u8) -> Vec<Clause> {
    let mut result = Vec::new();
    let mut buffer = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '#' && chars.peek().is_some_and(|ch| matches!(ch, '1' | '2' | '3')) {
            voice = chars.next().unwrap() as u8 - b'0';
            continue;
        }
        let newline = ch == '\r' || ch == '\n';
        if ch == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
        }
        if !newline {
            buffer.push(ch);
        }
        if newline
            || matches!(ch, ',' | '.' | '!' | '?' | ':' | ';' | '(' | ')' | '…')
                && !(ch == '.' && chars.peek() == Some(&'.'))
        {
            if !buffer.trim().is_empty() {
                result.push(Clause {
                    text: std::mem::take(&mut buffer),
                    voice,
                });
            } else {
                buffer.clear();
            }
        }
    }
    if !buffer.trim().is_empty() {
        result.push(Clause {
            text: buffer,
            voice,
        });
    }
    result
}
