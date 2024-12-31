use bitvec::prelude::BitVec;

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

pub fn from(bit_vec: &BitVec, option: &Option) -> String {
    match option.format {
        Format::Binary => binary(bit_vec, option.width, option.twos_complement),
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

fn binary(bit_vec: &BitVec, width: usize, twos_complement: bool) -> String {
    let bit_vec = get_resized_bitvec(bit_vec, width, twos_complement);
    String::from_iter(bit_vec.iter().rev().map(|b| if *b { "1" } else { "0" }))
}

fn get_resized_bitvec(bit_vec: &BitVec, width: usize, twos_complement: bool) -> BitVec {
    let mut bit_vec = bit_vec.clone();
    let fills = if twos_complement {
        *bit_vec.first().as_deref().unwrap_or(&false)
    } else {
        false
    };
    bit_vec.resize(width, fills);
    bit_vec
}
