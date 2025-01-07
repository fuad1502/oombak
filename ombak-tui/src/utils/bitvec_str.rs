use bitvec::{order::Lsb0, prelude::BitVec};

use crate::components::models;

#[derive(Clone)]
pub struct Option {
    pub format: Format,
    pub width: usize,
    pub twos_complement: bool,
}

#[derive(Copy, Clone)]
pub enum Format {
    Binary,
}

pub fn from(bit_vec: &BitVec<u32>, option: &Option) -> String {
    match option.format {
        Format::Binary => binary(bit_vec, option.width, option.twos_complement),
    }
}

pub fn parse(value: &str) -> Result<BitVec<u32>, String> {
    let chars: Vec<char> = value.chars().collect();
    if chars.is_empty() {
        return Err("cannot parse empty value".to_string());
    }
    if chars[0] == '0' && chars.len() > 1 {
        parse_non_decimal(&chars)
    } else {
        parse_decimal(&chars)
    }
}

impl Default for Option {
    fn default() -> Self {
        Self {
            format: Format::Binary,
            width: 0,
            twos_complement: false,
        }
    }
}

impl From<&models::WaveSpec> for Option {
    fn from(wave_spec: &models::WaveSpec) -> Self {
        Self {
            width: wave_spec.wave.width,
            format: wave_spec.format,
            twos_complement: wave_spec.signed,
        }
    }
}

fn binary(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> String {
    let bit_vec = get_resized_bitvec(bit_vec, width, twos_complement);
    String::from_iter(bit_vec.iter().rev().map(|b| if *b { "1" } else { "0" }))
}

fn get_resized_bitvec(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> BitVec<u32> {
    let mut bit_vec = bit_vec.clone();
    let fills = if twos_complement {
        *bit_vec.first().as_deref().unwrap_or(&false)
    } else {
        false
    };
    bit_vec.resize(width, fills);
    bit_vec
}

fn parse_non_decimal(chars: &[char]) -> Result<BitVec<u32>, String> {
    match chars[1] {
        'b' => parse_binary(&chars[2..]),
        c => Err(format!("unknown radix identifier '0{c}'")),
    }
}

fn parse_binary(chars: &[char]) -> Result<BitVec<u32>, String> {
    let mut result = bitvec::bitvec![u32, Lsb0;];
    for c in chars.iter().rev() {
        match c {
            '0' => result.push(false),
            '1' => result.push(true),
            _ => return Err("binary value can only contain 1's and 0's".to_string()),
        }
    }
    Ok(result)
}

fn parse_decimal(_chars: &[char]) -> Result<BitVec<u32>, String> {
    Err("parsing decimal value not yet supported".to_string())
}
