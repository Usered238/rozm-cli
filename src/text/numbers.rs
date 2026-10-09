//! Ukrainian cardinal forms recovered from Rozm's numeric string tables.

const SMALL: [&str; 20] = [
    "",
    "один",
    "два",
    "три",
    "чотири",
    "п'ять",
    "шість",
    "сім",
    "вісім",
    "дев'ять",
    "десять",
    "одинадцять",
    "дванадцять",
    "тринадцять",
    "чотирнадцять",
    "п'ятнадцять",
    "шістнадцять",
    "сімнадцять",
    "вісімнадцять",
    "дев'ятнадцять",
];
const TENS: [&str; 10] = [
    "",
    "",
    "двадцять",
    "тридцять",
    "сорок",
    "п'ятдесят",
    "шістдесят",
    "сімдесят",
    "вісімдесят",
    "дев'яносто",
];
const HUNDREDS: [&str; 10] = [
    "",
    "сто",
    "двісті",
    "триста",
    "чотириста",
    "п'ятсот",
    "шістсот",
    "сімсот",
    "вісімсот",
    "дев'ятсот",
];
const MAGNITUDES: [[&str; 3]; 9] = [
    ["тисяча", "тисячі", "тисяч"],
    ["мільйон", "мільйони", "мільйонів"],
    ["мільярд", "мільярди", "мільярдів"],
    ["трильйон", "трильйони", "трильйонів"],
    ["квадрильйон", "квадрильйони", "квадрильйонів"],
    ["квінтальйон", "квінтальйони", "квінтальйонів"],
    ["секстальйон", "секстальйони", "секстальйонів"],
    ["септальйон", "септальйони", "септальйонів"],
    ["октальйон", "октальйони", "октальйонів"],
];

fn words(digits: &str) -> String {
    if digits.len() > 30 {
        return "багато".into();
    }
    if digits.bytes().all(|b| b == b'0') {
        return "нуль".into();
    }
    let mut groups = Vec::new();
    let mut end = digits.len();
    let mut magnitude = 0;
    while end > 0 {
        let start = end.saturating_sub(3);
        let value = digits[start..end].parse::<usize>().unwrap();
        if value > 0 {
            let mut tokens = Vec::new();
            let remainder = value % 100;
            if value >= 100 {
                tokens.push(HUNDREDS[value / 100]);
            }
            if remainder >= 20 {
                tokens.push(TENS[remainder / 10]);
            }
            let unit = if remainder >= 20 {
                remainder % 10
            } else {
                remainder
            };
            if unit > 0 {
                tokens.push(if magnitude == 1 && unit == 1 {
                    "одна"
                } else if magnitude == 1 && unit == 2 {
                    "дві"
                } else {
                    SMALL[unit]
                });
            }
            if magnitude > 0 {
                let form = if unit == 1 {
                    0
                } else if (2..=4).contains(&unit) {
                    1
                } else {
                    2
                };
                tokens.push(MAGNITUDES[magnitude - 1][form]);
            }
            groups.push(tokens.join(" "));
        }
        end = start;
        magnitude += 1;
    }
    groups.reverse();
    groups.join(" ")
}

pub(super) fn expand(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            let mut digits = String::from(ch);
            while chars.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                digits.push(chars.next().unwrap());
            }
            output.push(' ');
            output.push_str(&words(&digits));
            output.push(' ');
        } else {
            output.push(ch);
        }
    }
    output
}
