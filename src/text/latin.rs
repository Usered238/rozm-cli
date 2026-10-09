//! Rozm's Latin fallback transliteration and letter-name tables.

const LETTERS: [&str; 26] = [
    "а", "б", "к", "д", "е", "ф", "г", "г", "і", "дж", "к", "л", "м", "н", "о", "п", "к", "р", "с",
    "т", "у", "в", "в", "кс", "і", "з",
];
const NAMES: [&str; 26] = [
    "ей", "бі", "сі", "ді", "і", "еф", "джі", "ейч", "ай", "джей", "кей", "ель", "ем", "ен", "оу",
    "пі", "ку", "ар", "ес", "ті", "ю", "ві", "дабл", "екс", "вай", "зет",
];

fn transliterate(word: &str) -> String {
    let mut output = String::new();
    if word.chars().filter(|ch| ch.is_ascii_uppercase()).count() > 1 {
        for b in word.bytes() {
            output.push_str(NAMES[(b.to_ascii_lowercase() - b'a') as usize]);
            output.push(' ');
        }
        return output;
    }
    let bytes = word.to_ascii_lowercase().into_bytes();
    let mut i = 0;
    let mut accent = false;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"sh") {
            output.push('ш');
            i += 2;
            continue;
        }
        if bytes[i..].starts_with(b"ch") {
            output.push('ч');
            i += 2;
            continue;
        }
        let syllable = if bytes[i] == b'c'
            && bytes
                .get(i + 1)
                .is_some_and(|b| matches!(b, b'e' | b'i' | b'y'))
        {
            "с"
        } else {
            LETTERS[(bytes[i] - b'a') as usize]
        };
        output.push_str(syllable);
        if !accent && syllable.starts_with(['а', 'е', 'и', 'і', 'о', 'у']) {
            output.push('\\');
            accent = true;
        }
        i += 1;
    }
    output
}

pub(super) fn expand(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_alphabetic() {
            let mut word = String::from(ch);
            while chars.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                word.push(chars.next().unwrap());
            }
            output.push_str(&transliterate(&word));
            output.push(' ');
        } else {
            output.push(ch);
        }
    }
    output
}
