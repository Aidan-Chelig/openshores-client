//! Case-sensitive name routing used by Au3dsGLSLObject::AverageWeights.
//! These are the native matching rules, not a general skeleton naming scheme.

pub fn native_body_is_bone(name: &[u8], bone: i32) -> bool {
    let contains = |needle: &[u8]| name.windows(needle.len()).any(|part| part == needle);
    let bone = bone as u32;
    if contains(b"Torso") {
        return bone < 7;
    }
    if contains(b"Head") {
        return bone == 7 || bone == 12;
    }
    if contains(b"Tail") {
        return bone.wrapping_sub(17) < 6;
    }
    // Native tests anatomical words in this order; Left is a substring too.
    let left = contains(b"Left");
    for (word, left_bones, right_bones) in [
        (b"Hand".as_slice(), [23, 29, 35], [26, 32, 38]),
        (b"ForeArm".as_slice(), [24, 30, 36], [27, 33, 39]),
        (b"UpperArm".as_slice(), [25, 31, 37], [28, 34, 40]),
        (b"Foot".as_slice(), [41, 47, 53], [44, 50, 56]),
        (b"Calf".as_slice(), [42, 48, 54], [45, 51, 57]),
        (b"Thigh".as_slice(), [43, 49, 55], [46, 52, 58]),
    ] {
        if contains(word) {
            return if left { left_bones } else { right_bones }.contains(&bone);
        }
    }
    contains(b"Jaw") && bone.wrapping_sub(59) < 2
}

/// Unknown names deliberately use the native base-tail fallback.
pub fn native_body_tail_bone(name: &[u8]) -> i32 {
    match name {
        b"Tail1" => 18,
        b"Tail2" => 19,
        b"Tail3" => 20,
        b"Tail4" => 21,
        b"Tail5" => 22,
        _ => 17,
    }
}

/// Unknown names deliberately map to zero, not the first neck bone.
pub fn native_body_neck_bone(name: &[u8]) -> i32 {
    match name {
        b"Neck00" => 8,
        b"Neck10" => 13,
        b"Neck01" => 9,
        b"Neck11" => 14,
        b"Neck02" => 10,
        b"Neck12" => 15,
        b"Neck03" => 11,
        b"Neck13" => 16,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_body_bone_names_preserve_priority_case_and_full_integer_bounds() {
        assert!(native_body_is_bone(b"LeftHand", 35));
        assert!(native_body_is_bone(b"Hand", 38));
        assert!(!native_body_is_bone(b"LeftHand", 38));
        assert!(!native_body_is_bone(b"lefthand", 23));
        assert!(native_body_is_bone(b"TorsoLeftHand", 0));
        assert!(!native_body_is_bone(b"TorsoLeftHand", 23));
        for bone in [-1, 87, 279, i32::MAX, i32::MIN] {
            assert!(!native_body_is_bone(b"LeftHand", bone));
        }
        assert!(native_body_is_bone(b"Tail", 22));
        assert!(!native_body_is_bone(b"Tail", 23));
        assert!(native_body_is_bone(b"Jaw", 60));
        assert!(!native_body_is_bone(b"Jaw", 61));
    }

    #[test]
    fn native_body_bone_names_exact_tail_and_neck_defaults() {
        for index in 0..6 {
            assert_eq!(
                native_body_tail_bone(format!("Tail{index}").as_bytes()),
                17 + index
            );
        }
        assert_eq!(native_body_tail_bone(b"prefixTail5"), 17);
        for (name, bone) in [
            ("Neck00", 8),
            ("Neck10", 13),
            ("Neck01", 9),
            ("Neck11", 14),
            ("Neck02", 10),
            ("Neck12", 15),
            ("Neck03", 11),
            ("Neck13", 16),
        ] {
            assert_eq!(native_body_neck_bone(name.as_bytes()), bone);
        }
        assert_eq!(native_body_neck_bone(b"Neck04"), 0);
        assert_eq!(native_body_neck_bone(b"neck00"), 0);
    }
}
