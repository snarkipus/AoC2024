use crate::keypads::{Key, Keypad};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumericValue {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    A,
    Blank,
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Zero => '0',
            Self::A => 'A',
            Self::Blank => ' ',
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumericKey(NumericValue);

impl fmt::Display for NumericKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Key for NumericKey {
    type Value = NumericValue;

    fn value(&self) -> Self::Value {
        self.0
    }

    fn from_char(c: char) -> Option<Self> {
        let value = match c {
            '1' => NumericValue::One,
            '2' => NumericValue::Two,
            '3' => NumericValue::Three,
            '4' => NumericValue::Four,
            '5' => NumericValue::Five,
            '6' => NumericValue::Six,
            '7' => NumericValue::Seven,
            '8' => NumericValue::Eight,
            '9' => NumericValue::Nine,
            '0' => NumericValue::Zero,
            'A' => NumericValue::A,
            ' ' => NumericValue::Blank,
            _ => return None,
        };
        Some(Self(value))
    }

    fn to_char(&self) -> char {
        match self.0 {
            NumericValue::One => '1',
            NumericValue::Two => '2',
            NumericValue::Three => '3',
            NumericValue::Four => '4',
            NumericValue::Five => '5',
            NumericValue::Six => '6',
            NumericValue::Seven => '7',
            NumericValue::Eight => '8',
            NumericValue::Nine => '9',
            NumericValue::Zero => '0',
            NumericValue::A => 'A',
            NumericValue::Blank => ' ',
        }
    }
}

pub fn create_numeric_keypad() -> Keypad<NumericKey> {
    let keys = vec![
        vec![
            NumericKey(NumericValue::Seven),
            NumericKey(NumericValue::Eight),
            NumericKey(NumericValue::Nine),
        ],
        vec![
            NumericKey(NumericValue::Four),
            NumericKey(NumericValue::Five),
            NumericKey(NumericValue::Six),
        ],
        vec![
            NumericKey(NumericValue::One),
            NumericKey(NumericValue::Two),
            NumericKey(NumericValue::Three),
        ],
        vec![
            NumericKey(NumericValue::Blank),
            NumericKey(NumericValue::Zero),
            NumericKey(NumericValue::A),
        ],
    ];

    Keypad::new(keys, |k| k.value() == NumericValue::Blank)
}
