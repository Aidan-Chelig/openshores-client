use super::*;

pub(super) const PROFILES: [FloatEncoding; 3] = [
    FloatEncoding::TypeWidths,
    FloatEncoding::SinglePrecision,
    FloatEncoding::DoublePrecision,
];
pub(super) fn float(out: &mut Vec<u8>, value: f64, profile: FloatEncoding) {
    if profile == FloatEncoding::DoublePrecision {
        out.extend(value.to_be_bytes());
    } else {
        out.extend((value as f32).to_be_bytes());
    }
}
pub(super) fn unit(profile: FloatEncoding) -> Vec<u8> {
    let mut out = vec![7];
    out.extend(u32::MAX.to_be_bytes());
    out.extend((-5_i32).to_be_bytes());
    out.push(0xfe);
    out.extend([0, 1, 8]);
    out.extend((-999_i64).to_be_bytes()); // one Fire condition
    for v in [1.0000000001, -0., -7.25] {
        float(&mut out, v, profile);
    }
    out
}
pub(super) fn dna(animal: bool) -> (Vec<u8>, NativeDna) {
    if animal {
        (
            vec![0xff, 0, 0, 0, 0],
            NativeDna::Au(NativeAuDna {
                version: 0xff,
                chromatids: [vec![], vec![]],
            }),
        )
    } else {
        let mut raw = [0xff; 24];
        raw[5] = 0x7f;
        raw[23] = 0;
        let mut out = 24_u32.to_be_bytes().to_vec();
        out.extend([0xff; 24]);
        (out, NativeDna::Dh(NativeDhDna(raw)))
    }
}
pub(super) fn fixture(flags: u16, profile: FloatEncoding, packed: u8) -> Vec<u8> {
    let mut out = unit(profile);
    out.extend(flags.to_be_bytes());
    if flags & 1 != 0 {
        out.push(packed);
        out.extend(dna(flags & 0x100 != 0).0);
    }
    if flags & 2 != 0 {
        float(&mut out, 1.0000000001, profile);
    }
    if flags & 4 != 0 {
        out.extend(i16::MIN.to_be_bytes());
    }
    if flags & 8 != 0 {
        out.extend([0, 1, 2, 3, 0x80, 0xff, 6, 7, 8, 0x8c]);
        out.extend(0xff81_u16.to_be_bytes());
    }
    if flags & 0x10 != 0 {
        out.extend([0x21, 3, 0xfd, 0xca, 0xfe]);
    }
    out.push(0xc3);
    if flags & 0x20 != 0 {
        out.push(4);
    }
    if flags & 0x80 != 0 {
        out.extend([1, 0, 0, 1, 0]);
    } // plain item, commodity1, zero byte20
    if flags & 0x200 != 0 {
        out.extend([0xff, 0x80]);
    }
    out
}

pub(super) fn expected_events(flags: u16, packed: u8, created: bool, now: i64) -> Vec<Event> {
    let kind = if flags & 0x100 != 0 {
        NativeCreatureBodyKind::Animalia
    } else {
        NativeCreatureBodyKind::DpBody
    };
    let mut out = vec![Event::Select(kind, created)];
    if flags & 1 != 0 {
        if kind == NativeCreatureBodyKind::DpBody {
            out.push(Event::Configure([
                u8::from(packed & 4 != 0),
                u8::from(packed & 1 == 0),
            ]));
        }
        out.push(Event::Dna(dna(flags & 0x100 != 0).1, packed));
    }
    if flags & 2 != 0 {
        out.push(Event::Scalar(1.));
    }
    if flags & 4 != 0 {
        out.push(Event::Hp(kind, -30));
    }
    out.push(Event::OldConsole);
    if flags & 8 != 0 {
        out.push(Event::Pose(
            kind,
            NativeCreaturePose {
                fields: [0, 1, 2, 3, 0x80, 0xff, 6, 7, 8, 0x8c],
                station: 0xff81,
            },
        ));
    }
    out.push(Event::Console(
        if flags & 0x10 != 0 { Some(0x21) } else { None },
        0x3d,
    ));
    out.push(Event::Clock);
    out.push(Event::Timestamp(
        kind,
        if flags & 0x40 != 0 {
            now.wrapping_sub(86_400_000)
        } else {
            now
        },
    ));
    out.push(Event::Seat);
    out
}
