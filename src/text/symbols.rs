//! Rozm's symbol names, expanded before cardinal numbers.
pub(super) fn expand(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    let letters = "АБВГДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЮЯ";
    let names = [
        r"а\", r"бе\", r"ве\", r"ге\", r"де\", r"е\", r"є\", r"же\", r"зе\", r"и\", r"і\", r"ї\",
        r"и\й", r"ка\", r"е\л", r"е\м", r"е\н", r"о\", r"пе\", r"е\р", r"е\с", r"те\", r"у\",
        r"е\ф", r"ха\", r"це\", r"ча\", r"ша\", r"ща\", r"ю\", r"я\",
    ];
    while let Some(ch) = chars.next() {
        if chars.peek() == Some(&'.')
            && let Some(index) = letters.chars().position(|letter| letter == ch)
        {
            output.push_str(names[index]);
            continue;
        }
        match ch {
            '%' => output.push_str("відсотків"),
            '№' => output.push_str("номер"),
            '+' => output.push_str("плюс"),
            '=' => output.push_str("дорівнює"),
            '*' => output.push_str("помножити на"),
            '/' => output.push_str("ділити на"),
            _ => output.push(ch),
        }
    }
    output
}
