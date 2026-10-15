//! Concrete DpTail::CurrentConnectLoc; other part methods remain unsupported.
use super::{NativeDpBoneTransforms, NativeDpPartKind, NativeDpPartState};
use std::io;

impl NativeDpPartState {
    /// Caller must pass this part's actual owner bones, not world/atom matrices.
    /// Native Tail always uses body+0x11f8 = slot17, even for a multi-bone tail.
    pub fn tail_current_connect_loc(&self, owner: &NativeDpBoneTransforms) -> io::Result<[f64; 3]> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "current attachment projection implemented for native Tail only",
            ));
        }
        Ok(owner.slots[17].project(self.connect_30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{native_body::NativeDpAllocatedPart, native_dna::NativeDhDna};

    #[test]
    fn tail_connection_projects_stored_attachment_through_slot_seventeen_only() {
        let mut part = NativeDpPartState {
            allocation: NativeDpAllocatedPart {
                kind: NativeDpPartKind::Tail,
                index: 0,
                dna: NativeDhDna([0; 24]),
            },
            connect_30: [1., 2., 3.],
            joints: super::super::NativeDpPartJoints::None,
        };
        let mut bones = NativeDpBoneTransforms::initialized();
        bones.slots[0].translate([100.; 3]);
        bones.slots[16].translate([200.; 3]);
        bones.slots[17].translate([10., 20., 30.]);
        bones.slots[18].translate([300.; 3]);
        let before = bones.clone();
        for raw14 in [0x10, 0x50, 0x90, 0xd0] {
            part.allocation.dna.0[14] = raw14;
            assert_eq!(
                part.tail_current_connect_loc(&bones).unwrap(),
                [11., 22., 33.]
            );
        }
        assert_eq!(bones, before);
        part.allocation.kind = NativeDpPartKind::HeadAndNeck;
        assert_eq!(
            part.tail_current_connect_loc(&bones).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
