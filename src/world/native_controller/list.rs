use super::NativeController;
use crate::protocol::QtReader;
use std::io;

const MAX_CONTROLLERS: usize = 20_000;

/// Nullable storage after RxUpdateBase's Append/empty-list disposal stage.
#[derive(Clone, Debug, Default)]
pub struct NativeControllerList {
    allocated: bool,
    entries: Vec<NativeController>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeControllerListReceive {
    pub count: u8,
    pub appended: usize,
    pub unsupported_types: Vec<u8>,
    pub consumed_bytes: usize,
}
impl NativeControllerList {
    pub fn allocated(&self) -> bool {
        self.allocated
    }
    pub fn entries(&self) -> &[NativeController] {
        &self.entries
    }

    /// Receive the count/types and bodies, then dispose empty storage as the
    /// base caller does. No deduplication, replacement, application or logging
    /// side effect is invented. Unknown types are reported in wire order.
    /// Earlier appends survive later errors; the Rust cap is not native policy.
    pub fn receive_base_layer(
        &mut self,
        input: &mut QtReader<'_>,
    ) -> io::Result<NativeControllerListReceive> {
        self.receive_bounded(input, MAX_CONTROLLERS)
    }
    fn receive_bounded(
        &mut self,
        input: &mut QtReader<'_>,
        limit: usize,
    ) -> io::Result<NativeControllerListReceive> {
        self.allocated = true;
        let length = input.remaining();
        let count = input.read_u8()?;
        let mut appended = 0;
        let mut unsupported_types = vec![];
        for _ in 0..count {
            let kind = input.read_u8()?;
            let Some(controller) = NativeController::read(kind, input)? else {
                unsupported_types.push(kind);
                continue;
            };
            if self.entries.len() >= limit {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native controller list exceeds budget",
                ));
            }
            self.entries.push(controller);
            appended += 1;
        }
        if self.entries.is_empty() {
            self.allocated = false;
        }
        Ok(NativeControllerListReceive {
            count,
            appended,
            unsupported_types,
            consumed_bytes: length - input.remaining(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn append_keeps_duplicates_unknown_types_and_body_boundary() {
        let mut list = NativeControllerList::default();
        let mut input = QtReader::new(&[4, 2, 255, 2, 5, 0xde]);
        let result = list.receive_base_layer(&mut input).unwrap();
        assert_eq!(result.appended, 2);
        assert_eq!(result.unsupported_types, [255, 5]);
        assert_eq!(
            list.entries,
            [NativeController::Motion, NativeController::Motion]
        );
        assert_eq!(input.remaining_bytes(), &[0xde]);
        assert!(list.allocated());
        list.receive_base_layer(&mut QtReader::new(&[0])).unwrap();
        assert_eq!(list.entries.len(), 2);
        assert!(list.allocated());
    }
    #[test]
    fn empty_storage_is_disposed_only_after_completed_reception() {
        let mut list = NativeControllerList::default();
        for payload in [&[0][..], &[2, 5, 255][..]] {
            list.receive_base_layer(&mut QtReader::new(payload))
                .unwrap();
            assert!(!list.allocated());
            assert!(list.entries.is_empty());
        }
        assert!(list.receive_base_layer(&mut QtReader::new(&[1])).is_err());
        assert!(list.allocated());
    }
    #[test]
    fn failures_retain_prior_appends_and_unknown_body_is_not_guessed() {
        let mut list = NativeControllerList::default();
        let mut input = QtReader::new(&[2, 2, 1, 0xde]);
        assert!(list.receive_base_layer(&mut input).is_err());
        assert_eq!(list.entries, [NativeController::Motion]);
        assert_eq!(input.remaining_bytes(), &[0xde]);
        assert!(
            list.receive_bounded(&mut QtReader::new(&[1, 2]), 1)
                .is_err()
        );
        assert_eq!(list.entries.len(), 1);
        let mut bytes = vec![255];
        bytes.extend([2; 255]);
        let outcome = NativeControllerList::default()
            .receive_base_layer(&mut QtReader::new(&bytes))
            .unwrap();
        assert_eq!(outcome.appended, 255);
    }
}
