use super::NativeColorQt7;
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeGene {
    pub kind: u8,
    pub color_08: NativeColorQt7,
    pub field_18: u16,
    pub suffix: NativeGeneSuffix,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeGeneSuffix {
    Feathers { field_1a: u8 },
    Hair { field_20: i64 },
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeGeneChain(pub Vec<NativeGene>);

pub(super) fn spend(budget: &mut usize) -> io::Result<()> {
    *budget = budget.checked_sub(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "native DNA element budget exceeded",
        )
    })?;
    Ok(())
}

impl NativeGeneChain {
    pub(super) fn read(input: &mut QtReader<'_>, budget: &mut usize) -> io::Result<Self> {
        let mut genes = Vec::new();
        for _ in 0..input.read_u16()? {
            spend(budget)?;
            let kind = input.read_u8()?;
            // Native factory type 4 and all higher types return null.
            if kind > 3 {
                continue;
            }
            let color_08 = NativeColorQt7::read(input)?;
            let field_18 = input.read_u16()?;
            let suffix = match kind {
                0 => NativeGeneSuffix::Feathers {
                    field_1a: input.read_u8()?,
                },
                1 => NativeGeneSuffix::Hair {
                    field_20: input.read_i64()?,
                },
                2 | 3 => NativeGeneSuffix::None,
                _ => unreachable!(),
            };
            genes.push(NativeGene {
                kind,
                color_08,
                field_18,
                suffix,
            });
        }
        Ok(Self(genes))
    }
}
