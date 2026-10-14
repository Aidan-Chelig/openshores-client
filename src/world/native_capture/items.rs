//! Item-field replay in capture order, not a complete native world snapshot.
use super::super::{native_atom::NativeAtomKind, native_item::NativeItemState};
use super::NativeCapture;
use std::{collections::BTreeMap, io};

#[derive(Clone, Debug, Default)]
pub struct NativeItemReplay {
    pub items: BTreeMap<u32, NativeItemState>,
    pub decoded_updates: usize,
    pub opaque_updates: usize,
}

impl NativeCapture {
    /// Retain established item effects. Conservative observation policy:
    /// unparsed item updates invalidate known
    /// fields; other atom kinds with the same ID remove the prior item record.
    /// No native timestamp filtering, deletion/visibility or hierarchy claims.
    /// Malformed known bodies fail before returning any partial replay.
    pub fn replay_items(&self) -> io::Result<NativeItemReplay> {
        let mut replay = NativeItemReplay::default();
        for frame in &self.frames {
            let Some((kind, base)) = frame.prefix() else {
                continue;
            };
            if *kind != NativeAtomKind::Item {
                replay.items.remove(&base.id);
                continue;
            }
            let state = replay.items.entry(base.id).or_default();
            match frame.item_body()? {
                Some(body) => {
                    state.apply(body);
                    replay.decoded_updates += 1;
                }
                None => {
                    state.invalidate_opaque();
                    replay.opaque_updates += 1;
                }
            }
        }
        Ok(replay)
    }
}
