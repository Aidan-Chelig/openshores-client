use super::{NativeChromosomeBody, gene::NativeGeneChain};
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeChromosomeField {
    pub offset: u8,
    pub value: NativeChromosomeValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeChromosomeValue {
    Byte(u8),
    Short(u16),
    Unsigned32(u32),
}

#[derive(Clone, Copy)]
enum Wire {
    B,
    S,
    WB,
    WS,
}

pub(super) fn read(
    kind: u8,
    input: &mut QtReader<'_>,
    budget: &mut usize,
) -> io::Result<NativeChromosomeBody> {
    use Wire::*;
    let genes = if input.read_u8()? & 1 != 0 {
        Some(NativeGeneChain::read(input, budget)?)
    } else {
        None
    };
    // Exact wire order from the factory-bound reader evidence. No field names
    // or signed-short semantics are inferred from biological class names.
    let layout: &[(u8, Wire)] = match kind {
        8 => &[
            (0x10, WB),
            (0x14, S),
            (0x16, S),
            (0x18, S),
            (0x1a, S),
            (0x1c, S),
            (0x1e, S),
            (0x20, S),
            (0x22, S),
        ],
        9 => &[
            (0x10, S),
            (0x12, S),
            (0x14, S),
            (0x16, S),
            (0x18, S),
            (0x1a, S),
            (0x1c, S),
            (0x1e, S),
        ],
        10 => &[
            (0x10, S),
            (0x12, S),
            (0x14, S),
            (0x16, B),
            (0x18, S),
            (0x1a, S),
            (0x1c, B),
            (0x1e, S),
            (0x20, B),
            (0x22, S),
        ],
        11 => &[
            (0x10, S),
            (0x12, S),
            (0x14, S),
            (0x16, S),
            (0x18, S),
            (0x1a, S),
            (0x1c, S),
            (0x1e, S),
            (0x20, S),
            (0x22, B),
            (0x24, S),
            (0x26, B),
        ],
        12 => &[(0x10, S), (0x14, WB), (0x18, B), (0x19, B)],
        13 => &[(0x10, WB), (0x14, B)],
        14 => &[(0x10, S), (0x14, WB), (0x18, B)],
        17 => &[(0x14, WB), (0x18, B), (0x10, S), (0x12, S)],
        18 => &[(0x10, WS), (0x14, S), (0x18, WB), (0x1c, S), (0x1e, B)],
        19 => &[(0x10, S), (0x12, S), (0x14, S), (0x16, S)],
        20 => &[
            (0x10, S),
            (0x12, S),
            (0x14, S),
            (0x16, S),
            (0x18, WB),
            (0x1c, S),
            (0x1e, S),
            (0x20, S),
        ],
        _ => unreachable!("only gene-bearing chromosome types"),
    };
    let mut fields = Vec::with_capacity(layout.len());
    for &(offset, wire) in layout {
        let value = match wire {
            B => NativeChromosomeValue::Byte(input.read_u8()?),
            S => NativeChromosomeValue::Short(input.read_u16()?),
            WB => NativeChromosomeValue::Unsigned32(u32::from(input.read_u8()?)),
            WS => NativeChromosomeValue::Unsigned32(u32::from(input.read_u16()?)),
        };
        fields.push(NativeChromosomeField { offset, value });
    }
    Ok(NativeChromosomeBody::WithGenes { genes, fields })
}
