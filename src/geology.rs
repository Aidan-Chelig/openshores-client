//! Source-established natural-feature records; no terrain deformation is inferred.
use crate::protocol::QtReader;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FeatureKind {
    Valley = 1,
    Crater,
    Mesa,
    Volcano,
    River,
    Chasm,
    Henge,
    Vent,
    Fissure,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    pub kind: FeatureKind,
    pub latitude: f32,
    pub longitude: f32,
    pub end: Option<(f32, f32)>,
    /// Source field names; physical meaning is not yet established for every kind.
    pub parameters: [f32; 4],
    pub henge: Option<Henge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Henge {
    pub flags: u8,
    pub slabs: i8,
    pub byte2: u8,
    pub auid: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geology {
    pub declared_count: u8,
    pub features: Vec<Feature>,
    /// Unknown records have no length prefix: subsequent fields cannot be located.
    pub unknown_kind: Option<u8>,
}

fn scalar(input: &mut QtReader<'_>) -> io::Result<f32> {
    let value = input.read_f32()?;
    if !value.is_finite() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "non-finite geology field",
        ));
    }
    Ok(value)
}

fn coordinate(input: &mut QtReader<'_>) -> io::Result<(f32, f32)> {
    // GlobeLLF's wire order is longitude, latitude.
    let longitude = scalar(input)?;
    Ok((scalar(input)?, longitude))
}

impl Geology {
    /// Stops at an unknown kind without interpreting its body or later fields.
    /// The caller retains the original packet body for lossless inspection.
    pub fn read_prefix(input: &mut QtReader<'_>) -> io::Result<Self> {
        let declared_count = input.read_u8()?;
        let mut result = Self {
            declared_count,
            features: Vec::new(),
            unknown_kind: None,
        };
        for _ in 0..declared_count {
            let raw = input.read_u8()?;
            let kind = match raw {
                1 => FeatureKind::Valley,
                2 => FeatureKind::Crater,
                3 => FeatureKind::Mesa,
                4 => FeatureKind::Volcano,
                5 => FeatureKind::River,
                6 => FeatureKind::Chasm,
                7 => FeatureKind::Henge,
                8 => FeatureKind::Vent,
                9 => FeatureKind::Fissure,
                _ => {
                    result.unknown_kind = Some(raw);
                    break;
                }
            };
            let (latitude, longitude) = coordinate(input)?;
            let mut feature = Feature {
                kind,
                latitude,
                longitude,
                end: None,
                parameters: [0.0; 4],
                henge: None,
            };
            use FeatureKind::*;
            if kind == Henge {
                feature.henge = Some(self::Henge {
                    flags: input.read_u8()?,
                    slabs: input.read_u8()? as i8,
                    byte2: input.read_u8()?,
                    auid: input.read_u32()?,
                });
                feature.parameters[3] = scalar(input)?;
            } else {
                feature.parameters[0] = scalar(input)?;
                if matches!(kind, Mesa | Volcano | Vent | Crater | River) {
                    feature.parameters[1] = scalar(input)?;
                }
                if matches!(kind, Valley | Chasm | Fissure | River) {
                    feature.end = Some(coordinate(input)?);
                }
                if matches!(kind, Crater | Valley | Chasm | Fissure | River) {
                    feature.parameters[2] = scalar(input)?;
                }
                if matches!(kind, Chasm | Fissure) {
                    feature.parameters[1] = scalar(input)?;
                }
                if matches!(kind, Chasm | Fissure | River) {
                    feature.parameters[3] = scalar(input)?;
                }
            }
            result.features.push(feature);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_natural_kind_has_exact_boundary() {
        for (kind, length) in [
            (1, 24),
            (2, 20),
            (3, 16),
            (4, 16),
            (5, 32),
            (6, 32),
            (7, 19),
            (8, 16),
            (9, 32),
        ] {
            let mut bytes = vec![1, kind];
            bytes.extend(vec![0; length]);
            bytes.push(0xab);
            let mut reader = QtReader::new(&bytes);
            let geology = Geology::read_prefix(&mut reader).unwrap();
            assert_eq!(geology.features.len(), 1);
            assert_eq!(geology.unknown_kind, None);
            assert_eq!(reader.read_u8().unwrap(), 0xab);
            for truncated in 0..length + 2 {
                assert!(Geology::read_prefix(&mut QtReader::new(&bytes[..truncated])).is_err());
            }
        }
    }

    #[test]
    fn unknown_records_do_not_claim_complete_decode() {
        let mut reader = QtReader::new(&[2, 10, 0xab]);
        let geology = Geology::read_prefix(&mut reader).unwrap();
        assert_eq!(geology.declared_count, 2);
        assert_eq!(geology.unknown_kind, Some(10));
        assert!(geology.features.is_empty());
        assert_eq!(reader.read_u8().unwrap(), 0xab);
    }

    #[test]
    fn longitude_first_and_signed_henge_slabs() {
        let mut bytes = vec![1, 7];
        bytes.extend(2.0f32.to_be_bytes());
        bytes.extend(1.0f32.to_be_bytes());
        bytes.extend([0x80, 0xff, 3]);
        bytes.extend(42u32.to_be_bytes());
        bytes.extend(4.0f32.to_be_bytes());
        let geo = Geology::read_prefix(&mut QtReader::new(&bytes)).unwrap();
        let feature = &geo.features[0];
        assert_eq!((feature.latitude, feature.longitude), (1.0, 2.0));
        assert_eq!(feature.henge.as_ref().unwrap().slabs, -1);
        assert_eq!(feature.parameters[3], 4.0);
    }
}
