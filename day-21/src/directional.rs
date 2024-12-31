use crate::keypads::{Key, Keypad};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionalValue {
    Up,
    Down,
    Left,
    Right,
    A,
    Blank,
}

impl fmt::Display for DirectionalValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Self::Up => '^',
            Self::Down => 'v',
            Self::Left => '<',
            Self::Right => '>',
            Self::A => 'A',
            Self::Blank => ' ',
        };
        write!(f, "{}", c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectionalKey(DirectionalValue);

impl fmt::Display for DirectionalKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Key for DirectionalKey {
    type Value = DirectionalValue;

    fn value(&self) -> Self::Value {
        self.0
    }

    fn from_char(c: char) -> Option<Self> {
        let value = match c {
            '^' => DirectionalValue::Up,
            'v' => DirectionalValue::Down,
            '<' => DirectionalValue::Left,
            '>' => DirectionalValue::Right,
            'A' => DirectionalValue::A,
            ' ' => DirectionalValue::Blank,
            _ => return None,
        };
        Some(Self(value))
    }

    fn to_char(&self) -> char {
        match self.0 {
            DirectionalValue::Up => '^',
            DirectionalValue::Down => 'v',
            DirectionalValue::Left => '<',
            DirectionalValue::Right => '>',
            DirectionalValue::A => 'A',
            DirectionalValue::Blank => ' ',
        }
    }
}

pub fn create_directional_keypad() -> Keypad<DirectionalKey> {
    let keys = vec![
        vec![
            DirectionalKey(DirectionalValue::Blank),
            DirectionalKey(DirectionalValue::Up),
            DirectionalKey(DirectionalValue::A),
        ],
        vec![
            DirectionalKey(DirectionalValue::Left),
            DirectionalKey(DirectionalValue::Down),
            DirectionalKey(DirectionalValue::Right),
        ],
    ];

    Keypad::new(keys, |k| k.value() == DirectionalValue::Blank)
}
