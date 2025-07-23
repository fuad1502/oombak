use bitvec::{order::Lsb0, prelude::BitVec};

use crate::components::models;

#[derive(Clone)]
pub struct Option {
    pub radix: Radix,
    pub width: usize,
    pub twos_complement: bool,
}

#[derive(Copy, Clone)]
pub enum Radix {
    Binary,
    Hexadecimal,
    Octal,
    Decimal,
}

pub fn from(bit_vec: &BitVec<u32>, option: &Option) -> String {
    match option.radix {
        Radix::Binary => binary(bit_vec, option.width, option.twos_complement),
        Radix::Hexadecimal => hexadecimal(bit_vec, option.width, option.twos_complement),
        Radix::Octal => octal(bit_vec, option.width, option.twos_complement),
        Radix::Decimal => decimal(bit_vec, option.width, option.twos_complement),
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
            radix: Radix::Binary,
            width: 0,
            twos_complement: false,
        }
    }
}

impl From<&models::WaveSpec> for Option {
    fn from(wave_spec: &models::WaveSpec) -> Self {
        Self {
            width: wave_spec.wave.width,
            radix: wave_spec.radix,
            twos_complement: wave_spec.signed,
        }
    }
}

fn binary(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> String {
    let bit_vec = get_resized_bitvec(bit_vec, width, twos_complement);
    String::from_iter(bit_vec.iter().rev().map(|b| if *b { "1" } else { "0" }))
}

fn hexadecimal(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> String {
    let width = round_to_nearest_larger_multiple(width, 4);
    let bit_vec = get_resized_bitvec(bit_vec, width, twos_complement);
    let iter0 = bit_vec.iter().step_by(4);
    let iter1 = bit_vec.iter().skip(1).step_by(4);
    let iter2 = bit_vec.iter().skip(2).step_by(4);
    let iter3 = bit_vec.iter().skip(3).step_by(4);
    iter0
        .zip(iter1)
        .zip(iter2)
        .zip(iter3)
        .map(|(((d, c), b), a)| binary_tuple_to_hexadecimal_digit((*a, *b, *c, *d)))
        .rev()
        .collect()
}

fn octal(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> String {
    let width = round_to_nearest_larger_multiple(width, 3);
    let bit_vec = get_resized_bitvec(bit_vec, width, twos_complement);
    let iter0 = bit_vec.iter().step_by(3);
    let iter1 = bit_vec.iter().skip(1).step_by(3);
    let iter2 = bit_vec.iter().skip(2).step_by(3);
    iter0
        .zip(iter1)
        .zip(iter2)
        .map(|((c, b), a)| binary_tuple_to_octal_digit((*a, *b, *c)))
        .rev()
        .collect()
}

fn decimal(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> String {
    if (twos_complement && width > 127) || (!twos_complement && width > 128) {
        unimplemented!()
    }

    let bit_vec = get_resized_bitvec(bit_vec, 128, twos_complement);
    let value = u128_from_bitvec(&bit_vec);

    if twos_complement && *bit_vec.last().unwrap() {
        let (value, _) = (u128::MAX - value).overflowing_add(1);
        format!("-{value}")
    } else {
        value.to_string()
    }
}

fn get_resized_bitvec(bit_vec: &BitVec<u32>, width: usize, twos_complement: bool) -> BitVec<u32> {
    let mut bit_vec = bit_vec.clone();
    let fills = if twos_complement {
        *bit_vec.last().as_deref().unwrap_or(&false)
    } else {
        false
    };
    bit_vec.resize(width, fills);
    bit_vec
}

fn parse_non_decimal(chars: &[char]) -> Result<BitVec<u32>, String> {
    match chars[1] {
        'b' => parse_binary(&chars[2..]),
        'x' => parse_hexadecimal(&chars[2..]),
        'o' => parse_octal(&chars[2..]),
        c => Err(format!("unknown radix identifier '0{c}'")),
    }
}

fn parse_hexadecimal(chars: &[char]) -> Result<BitVec<u32>, String> {
    let chars = binary_chars_from_hexadecimal_chars(chars)?;
    parse_binary(&chars)
}

fn parse_octal(chars: &[char]) -> Result<BitVec<u32>, String> {
    let chars = binary_chars_from_octal_chars(chars)?;
    parse_binary(&chars)
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

fn parse_decimal(chars: &[char]) -> Result<BitVec<u32>, String> {
    parse_radix(chars, 10)
}

fn binary_chars_from_hexadecimal_chars(chars: &[char]) -> Result<Vec<char>, String> {
    Ok(chars
        .iter()
        .map(|c| hexadecimal_digit_to_binary_chars(*c))
        .collect::<Result<Vec<Vec<char>>, String>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn binary_chars_from_octal_chars(chars: &[char]) -> Result<Vec<char>, String> {
    Ok(chars
        .iter()
        .map(|c| octal_digit_to_binary_chars(*c))
        .collect::<Result<Vec<Vec<char>>, String>>()?
        .into_iter()
        .flatten()
        .collect())
}

fn hexadecimal_digit_to_binary_chars(c: char) -> Result<Vec<char>, String> {
    match c {
        '0' => Ok(vec!['0', '0', '0', '0']),
        '1' => Ok(vec!['0', '0', '0', '1']),
        '2' => Ok(vec!['0', '0', '1', '0']),
        '3' => Ok(vec!['0', '0', '1', '1']),
        '4' => Ok(vec!['0', '1', '0', '0']),
        '5' => Ok(vec!['0', '1', '0', '1']),
        '6' => Ok(vec!['0', '1', '1', '0']),
        '7' => Ok(vec!['0', '1', '1', '1']),
        '8' => Ok(vec!['1', '0', '0', '0']),
        '9' => Ok(vec!['1', '0', '0', '1']),
        'a' | 'A' => Ok(vec!['1', '0', '1', '0']),
        'b' | 'B' => Ok(vec!['1', '0', '1', '1']),
        'c' | 'C' => Ok(vec!['1', '1', '0', '0']),
        'd' | 'D' => Ok(vec!['1', '1', '0', '1']),
        'e' | 'E' => Ok(vec!['1', '1', '1', '0']),
        'f' | 'F' => Ok(vec!['1', '1', '1', '1']),
        _ => Err(format!("{c} is not a valid hexadecimal digit")),
    }
}

fn octal_digit_to_binary_chars(c: char) -> Result<Vec<char>, String> {
    match c {
        '0' => Ok(vec!['0', '0', '0']),
        '1' => Ok(vec!['0', '0', '1']),
        '2' => Ok(vec!['0', '1', '0']),
        '3' => Ok(vec!['0', '1', '1']),
        '4' => Ok(vec!['1', '0', '0']),
        '5' => Ok(vec!['1', '0', '1']),
        '6' => Ok(vec!['1', '1', '0']),
        '7' => Ok(vec!['1', '1', '1']),
        _ => Err(format!("{c} is not a valid octal digit")),
    }
}

fn binary_tuple_to_hexadecimal_digit(value: (bool, bool, bool, bool)) -> char {
    match value {
        (false, false, false, false) => '0',
        (false, false, false, true) => '1',
        (false, false, true, false) => '2',
        (false, false, true, true) => '3',
        (false, true, false, false) => '4',
        (false, true, false, true) => '5',
        (false, true, true, false) => '6',
        (false, true, true, true) => '7',
        (true, false, false, false) => '8',
        (true, false, false, true) => '9',
        (true, false, true, false) => 'A',
        (true, false, true, true) => 'B',
        (true, true, false, false) => 'C',
        (true, true, false, true) => 'D',
        (true, true, true, false) => 'E',
        (true, true, true, true) => 'F',
    }
}

fn binary_tuple_to_octal_digit(value: (bool, bool, bool)) -> char {
    match value {
        (false, false, false) => '0',
        (false, false, true) => '1',
        (false, true, false) => '2',
        (false, true, true) => '3',
        (true, false, false) => '4',
        (true, false, true) => '5',
        (true, true, false) => '6',
        (true, true, true) => '7',
    }
}

fn parse_radix(chars: &[char], radix: u32) -> Result<BitVec<u32>, String> {
    // TODO: support decimal numbers that does not fit into a 128-bit variable
    let num_str: String = chars.iter().collect();
    let num = u128::from_str_radix(&num_str, radix)
        .map_err(|e| format!("cannot parse {num_str} as radix-{radix} number: {e}"))?;
    let mut result = bitvec::bitvec![u32, Lsb0;];
    for i in 0..size_of::<u128>() {
        result.push(num >> i & 0b1 == 0b1);
    }
    Ok(result)
}

fn round_to_nearest_larger_multiple(value: usize, multiple: usize) -> usize {
    if value % multiple != 0 {
        (value / multiple + 1) * multiple + 1
    } else {
        value
    }
}

fn u128_from_bitvec(bitvec: &BitVec<u32>) -> u128 {
    bitvec
        .iter()
        .map(|b| if *b { 1 } else { 0 })
        .zip(0..)
        .map(|(v, shift)| v << shift)
        .sum()
}
