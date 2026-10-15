//! Concrete part CurrentConnectLoc and limb Bones out-parameter mappings.
use super::{NativeDpBoneTransforms, NativeDpPartKind, NativeDpPartState, native_dh_waist_vertex};
use std::io;

impl NativeDpPartState {
    /// Actual Bones(ref1,ref2,ref3) outputs, in native reference-argument order.
    /// Arms default to pair0 for any index other than1/2. Invalid legs leave
    /// native outputs unwritten, represented explicitly as Unsupported here.
    pub fn limb_bones(&self) -> io::Result<[usize; 3]> {
        use NativeDpPartKind::*;
        let index = self.allocation.index;
        let (base, pair) = match self.allocation.kind {
            LeftArmAndHand => (
                23,
                match index {
                    1 => 1,
                    2 => 2,
                    _ => 0,
                },
            ),
            RightArmAndHand => (
                26,
                match index {
                    1 => 1,
                    2 => 2,
                    _ => 0,
                },
            ),
            LeftLegAndFoot | RightLegAndFoot if index > 2 => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native leg Bones leaves outputs unwritten for this index",
                ));
            }
            LeftLegAndFoot => (41, index as usize),
            RightLegAndFoot => (44, index as usize),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "native limb Bones requested for a non-limb part",
                ));
            }
        };
        let first = base + pair * 6;
        Ok([first + 2, first + 1, first])
    }

    /// Actual part attachment projected through this part's owner bone state.
    /// Torso projects origin, ignoring its stored ConnectLoc; other parts use
    /// the stored connection. This is not world/environment positioning.
    pub fn current_connect_loc(&self, owner: &NativeDpBoneTransforms) -> io::Result<[f64; 3]> {
        use NativeDpPartKind::*;
        let slot = match self.allocation.kind {
            Torso => native_dh_waist_vertex(&self.allocation.dna).saturating_sub(1) as usize,
            Tail => 17,
            HeadAndNeck => {
                7 + usize::from(
                    (self.allocation.dna.0[14] & 3) + ((self.allocation.dna.0[5] >> 1) & 1),
                ) + usize::from(self.allocation.index == 1) * 5
            }
            LeftArmAndHand | RightArmAndHand | LeftLegAndFoot | RightLegAndFoot => {
                self.limb_bones()?[0]
            }
        };
        Ok(owner.slots[slot].project(if self.allocation.kind == Torso {
            [0.; 3]
        } else {
            self.connect_30
        }))
    }

    /// Caller must pass this part's actual owner bones, not world/atom matrices.
    /// Native Tail always uses body+0x11f8 = slot17, even for a multi-bone tail.
    pub fn tail_current_connect_loc(&self, owner: &NativeDpBoneTransforms) -> io::Result<[f64; 3]> {
        if self.allocation.kind != NativeDpPartKind::Tail {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "current attachment projection implemented for native Tail only",
            ));
        }
        self.current_connect_loc(owner)
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

#[cfg(test)]
mod reader_tests;
