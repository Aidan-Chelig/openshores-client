use super::NativeEffect;
use crate::protocol::QtReader;
use std::io;

const MAX_EFFECTS: usize = 20_000;

/// Native nullable effects storage; zero-count reception allocates but never
/// clears an existing list. Capacity bound is Rust safety, not a native limit.
#[derive(Clone, Debug, Default)]
pub struct NativeEffectList {
    allocated: bool,
    effects: Vec<NativeEffect>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeEffectListReceive {
    pub decoded: u8,
    pub appended: usize,
    pub consumed_bytes: usize,
}
impl NativeEffectList {
    pub fn allocated(&self) -> bool {
        self.allocated
    }
    pub fn effects(&self) -> &[NativeEffect] {
        &self.effects
    }

    /// One fresh temporary per list update, reused for every record, including
    /// duplicate records. Clock observations are supplied once per effect.
    /// Earlier completed records survive later decode/capacity errors. No
    /// renderer, expiry, global adjustment guess or Qt-error emulation.
    pub fn receive(
        &mut self,
        input: &mut QtReader<'_>,
        mut clock: impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeEffectListReceive> {
        self.receive_bounded(input, MAX_EFFECTS, &mut clock)
    }
    fn receive_bounded(
        &mut self,
        input: &mut QtReader<'_>,
        limit: usize,
        clock: &mut impl FnMut() -> (i64, i64),
    ) -> io::Result<NativeEffectListReceive> {
        let length = input.remaining();
        self.allocated = true;
        let count = input.read_u8()?;
        let mut value = NativeEffect::from_base_receive_temporary();
        let mut appended = 0;
        for _ in 0..count {
            let (now, adjustment) = clock();
            value.receive(input, now, adjustment)?;
            if self
                .effects
                .iter()
                .any(|existing| existing.matches_append_key(&value))
            {
                continue;
            }
            if self.effects.len() >= limit {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native effect list exceeds budget",
                ));
            }
            self.effects.push(value.clone());
            appended += 1;
        }
        Ok(NativeEffectListReceive {
            decoded: count,
            appended,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_temporary_is_reused_and_zero_count_retains_list() {
        let mut bytes = vec![3, 1, 0, 1];
        for value in [1_f32, 2., 3.] {
            bytes.extend(value.to_be_bytes());
        }
        bytes.extend([0, 0, 2, 0, 0, 2, 0xde]);
        let mut input = QtReader::new(&bytes);
        let mut list = NativeEffectList::default();
        let mut calls = 0;
        let outcome = list
            .receive(&mut input, || {
                calls += 1;
                (0, 0)
            })
            .unwrap();
        assert_eq!(calls, 3);
        assert_eq!(outcome.appended, 2);
        assert_eq!(list.effects[1].vector_28, [1., 2., 3.]);
        assert_eq!(input.remaining_bytes(), &[0xde]);
        list.receive(&mut QtReader::new(&[0]), || panic!("zero count clock"))
            .unwrap();
        assert_eq!(list.effects.len(), 2);
        assert!(list.allocated());
        // New list update resets its temporary instead of inheriting last packet.
        list.receive(&mut QtReader::new(&[1, 0, 0, 2]), || (0, 0))
            .unwrap();
        assert_eq!(list.effects.len(), 3);
        assert_eq!(list.effects[2].vector_28, [0.; 3]);
    }
    #[test]
    fn partial_decode_and_capacity_failure_retain_completed_records() {
        let mut list = NativeEffectList::default();
        let mut input = QtReader::new(&[2, 0, 0, 1, 0]);
        assert!(list.receive(&mut input, || (0, 0)).is_err());
        assert_eq!(list.effects.len(), 1);
        assert_eq!(input.remaining_bytes(), &[0]);
        let mut input = QtReader::new(&[2, 0, 0, 1, 0, 0, 2]);
        assert!(list.receive_bounded(&mut input, 1, &mut || (0, 0)).is_err());
        assert_eq!(list.effects.len(), 1);
        assert_eq!(input.remaining(), 0);
    }
    #[test]
    fn maximum_unsigned_count_is_not_a_signed_negative() {
        let mut bytes = vec![255];
        for _ in 0..255 {
            bytes.extend([0, 0, 0]);
        }
        let mut list = NativeEffectList::default();
        let mut calls = 0;
        let outcome = list
            .receive(&mut QtReader::new(&bytes), || {
                calls += 1;
                (0, 0)
            })
            .unwrap();
        assert_eq!(calls, 255);
        assert_eq!(outcome.decoded, 255);
        assert_eq!(outcome.appended, 1);
    }
}
