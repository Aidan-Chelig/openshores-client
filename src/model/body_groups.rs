//! Native body group classification, pivot extraction and position membership.
//! This does not apply group deformation, average bone weights or draw a mesh.
use super::{VertexGroup, VertexGroupChunk, invalid};
use std::{cmp::Ordering, collections::BTreeMap, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position([u32; 3]);
impl Position {
    fn new(point: [f32; 3]) -> io::Result<Self> {
        if point.iter().any(|v| v.is_nan()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native group NaN-key ordering is not reconstructed",
            ));
        }
        Ok(Self(point.map(|v| if v == 0. { 0 } else { v.to_bits() })))
    }
    fn point(self) -> [f32; 3] {
        self.0.map(f32::from_bits)
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in self.point().into_iter().zip(other.point()) {
            let order = a.total_cmp(&b);
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBodyVertexGroup {
    name: Vec<u8>,
    pub kind_34: u32,
    pub flags_50: [bool; 6],
    pub point_28: [f32; 3],
    weights: BTreeMap<Position, ([f32; 3], f64)>,
}

fn contains(name: &[u8], word: &[u8]) -> bool {
    name.windows(word.len()).any(|part| part == word)
}

fn classify(name: &[u8]) -> (u32, [bool; 6]) {
    // Case-sensitive byte-to-QChar names, in native precedence order. In
    // particular TopScalePt matches the EARLIER ScalePt substring test.
    let mut flags = [false; 6];
    for (word, kind) in [(b"ScalePt".as_slice(), 2), (b"ConnectPt", 3), (b"MidPt", 4)] {
        if contains(name, word) {
            return (kind, flags);
        }
    }
    if name == b"Top" {
        return (5, flags);
    }
    if name == b"Bottom" {
        return (6, flags);
    }
    if contains(name, b"BottomSkin") {
        return (7, flags);
    }
    for (word, kind) in [
        (b"TopConnect".as_slice(), 8),
        (b"BottomConnect", 9),
        (b"HandWingPt", 10),
        (b"EyePosition", 11),
        (b"TopScalePt", 12),
        (b"MidScalePt", 13),
        (b"BottomScalePt", 14),
    ] {
        if name == word {
            return (kind, flags);
        }
    }
    for (i, word) in [
        b"Torso".as_slice(),
        b"Hand",
        b"Foot",
        b"Neck",
        b"Tail",
        b"Head",
    ]
    .into_iter()
    .enumerate()
    {
        if contains(name, word) {
            flags[i] = true;
            return (1, flags);
        }
    }
    let kind = match name {
        b"SeatLoc" => 15,
        b"OffLoad" | b"Offload" => 16,
        b"WeaponLoc" => 17,
        b"DirLoc" => 18,
        b"WeaponUDPt" => 19,
        b"WeaponLRPt" => 20,
        b"Screen1" => 21,
        b"Screen2" => 22,
        b"FlagDown" | b"Flag Down" => 23,
        b"FlagUp" | b"Flag Up" => 24,
        b"Missile Control" | b"MissileControl" | b"Missle Control" => 25,
        b"Missile Deploy" | b"MissleDeploy" | b"MissileDeploy" => 26,
        b"Seat1" => 27,
        b"Seat2" => 28,
        b"Seat3" => 29,
        b"Stand1" => 30,
        b"Stand2" => 31,
        b"Stand3" => 32,
        b"Stand4" => 33,
        b"Turret 1" | b"Turret1" => 34,
        b"Turret 2" | b"Turret2" => 35,
        b"Turret 3" | b"Turret3" => 36,
        b"Turret 4" | b"Turret4" => 37,
        _ => 1,
    };
    (kind, flags)
}

impl NativeBodyVertexGroup {
    pub fn from_group(group: &VertexGroup) -> io::Result<Self> {
        if group
            .field_order
            .iter()
            .filter(|f| **f == VertexGroupChunk::Name)
            .count()
            != usize::from(group.name.is_some())
            || group
                .field_order
                .iter()
                .filter(|f| **f == VertexGroupChunk::Members)
                .count()
                != usize::from(group.members.is_some())
        {
            return Err(invalid("missing native body group field order"));
        }
        let mut result = Self {
            name: Vec::new(),
            kind_34: 1,
            flags_50: [false; 6],
            point_28: [0.; 3],
            weights: BTreeMap::new(),
        };
        for field in &group.field_order {
            match field {
                VertexGroupChunk::Name => {
                    result.name = group
                        .name
                        .clone()
                        .ok_or_else(|| invalid("missing group name"))?;
                    (result.kind_34, result.flags_50) = classify(
                        group
                            .name
                            .as_deref()
                            .ok_or_else(|| invalid("missing group name"))?,
                    );
                }
                VertexGroupChunk::Members => {
                    let members = group
                        .members
                        .as_ref()
                        .ok_or_else(|| invalid("missing group members"))?;
                    if members.len() > u16::MAX as usize {
                        return Err(invalid("native group record count exceeds u16"));
                    }
                    for member in members {
                        if !member.weight.is_finite()
                            || member.position.iter().any(|v| !v.is_finite())
                        {
                            return Err(invalid("nonfinite native body group record"));
                        }
                    }
                    if members.len() == 1 {
                        let kind = result.kind_34;
                        if matches!(kind, 3 | 10) || kind < 34 && (0x3f987f800_u64 >> kind) & 1 != 0
                        {
                            result.point_28 = members[0].position;
                        }
                        continue; // Single-record path NEVER inserts a weight.
                    }
                    for (i, member) in members.iter().enumerate() {
                        if i + 1 == members.len()
                            && (matches!(result.kind_34, 2..=6 | 10 | 11)
                                || result.flags_50.iter().any(|v| *v))
                        {
                            result.point_28 = member.position;
                            break;
                        }
                        let entry = result
                            .weights
                            .entry(Position::new(member.position)?)
                            .or_insert((member.position, 0.));
                        // QMap updates the value, not the original stored key.
                        entry.1 = f64::from(member.weight);
                    }
                }
            }
        }
        Ok(result)
    }

    pub fn name(&self) -> &[u8] {
        &self.name
    }

    /// ApplyScaleFactor constructs its lookup key by narrowing xyz to f32.
    /// Some(0) is still membership; it must not be treated as an absent key.
    pub fn matching_weight(&self, point: [f64; 3]) -> io::Result<Option<f64>> {
        Ok(self
            .weights
            .get(&Position::new(point.map(|v| v as f32))?)
            .map(|entry| entry.1))
    }

    pub fn entries(&self) -> impl Iterator<Item = ([f32; 3], f64)> + '_ {
        self.weights.values().copied()
    }

    /// Rebuild QMap keys in original numeric key order. Collisions keep the
    /// first stored key and the last assigned value, as QMap::insert does.
    pub(super) fn remap_positions(
        &self,
        mut transform: impl FnMut([f32; 3]) -> io::Result<[f32; 3]>,
    ) -> io::Result<Self> {
        let mut result = self.clone();
        result.weights.clear();
        for (point, weight) in self.entries() {
            let point = transform(point)?;
            result
                .weights
                .entry(Position::new(point)?)
                .or_insert((point, 0.))
                .1 = weight;
        }
        Ok(result)
    }

    pub(super) fn merge_weight_map_from(&mut self, source: &Self, replace: bool) {
        if replace {
            self.weights = source.weights.clone();
        } else {
            for (position, (point, weight)) in &source.weights {
                self.weights.entry(*position).or_insert((*point, 0.)).1 = *weight;
            }
        }
    }
}

#[cfg(test)]
mod tests;
