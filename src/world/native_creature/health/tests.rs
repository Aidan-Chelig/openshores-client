use super::*;

struct Parts {
    values: Vec<f32>,
    calls: Vec<usize>,
    count_calls: usize,
    fail: Option<usize>,
}
impl Parts {
    fn new(values: Vec<f32>) -> Self {
        Self {
            values,
            calls: vec![],
            count_calls: 0,
            fail: None,
        }
    }
}
impl NativePartVolumeContext for Parts {
    fn part_count(&mut self) -> io::Result<usize> {
        self.count_calls += 1;
        Ok(self.values.len())
    }
    fn part_volume(&mut self, index: usize) -> io::Result<f32> {
        self.calls.push(index);
        if self.fail == Some(index) {
            Err(io::ErrorKind::Unsupported.into())
        } else {
            Ok(self.values[index])
        }
    }
}
fn flags(a: u8, b: u8, c: u8) -> NativeDpHealthFlags {
    NativeDpHealthFlags {
        byte_19: a,
        byte_1a: b,
        byte_1d: c,
    }
}

#[test]
fn dp_float_cache_retention_zero_recomputation_flags_and_partial_failure() {
    let mut state = NativeDpHealthState::default();
    let mut parts = Parts::new(vec![0.25, 0.75]);
    let result = state.max_hit_points(flags(0, 0, 0), &mut parts).unwrap();
    assert_eq!(result.hit_points, 29);
    assert!(!result.negative_short_replaced);
    assert_eq!(state.cached_volume_68, 1.);
    assert_eq!(parts.calls, [0, 1]);
    parts.values = vec![1000.];
    let result = state.max_hit_points(flags(0, 0, 4), &mut parts).unwrap();
    assert_eq!(result.hit_points, 19);
    assert_eq!(parts.count_calls, 1);
    let result = state.max_hit_points(flags(3, 6, 4), &mut parts).unwrap();
    assert_eq!(result.hit_points, 14);
    let mut empty = Parts::new(vec![]);
    let mut zero_state = NativeDpHealthState::default();
    for _ in 0..2 {
        assert_eq!(
            zero_state
                .max_hit_points(flags(0, 0, 0), &mut empty)
                .unwrap()
                .hit_points,
            1
        );
    }
    assert_eq!(empty.count_calls, 2);
    let mut state = NativeDpHealthState::default();
    let mut parts = Parts::new(vec![1., 2.]);
    parts.fail = Some(1);
    assert_eq!(
        state
            .max_hit_points(flags(0, 0, 0), &mut parts)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(state.cached_volume_68, 1.);
    assert_eq!(parts.calls, [0, 1]);
    assert_eq!(
        state
            .max_hit_points(flags(0, 0, 0), &mut parts)
            .unwrap()
            .hit_points,
        29
    );
    assert_eq!(parts.calls, [0, 1]);
}

#[test]
fn dp_raw_flag_masks_overflow_diagnostics_and_model_budget_are_explicit() {
    let mut state = NativeDpHealthState {
        cached_volume_68: 1.,
    };
    let mut parts = Parts::new(vec![]);
    for a in 0..=255 {
        for b in 0..=255 {
            for c in [0, 4, 255] {
                let result = state.max_hit_points(flags(a, b, c), &mut parts).unwrap();
                let expected = if c & 4 == 0 {
                    29
                } else if a & 3 == 0 || b & 6 == 0 {
                    19
                } else {
                    14
                };
                assert_eq!(result.hit_points, expected);
                assert!(!result.negative_short_replaced);
            }
        }
    }
    assert_eq!(parts.count_calls, 0);
    state.cached_volume_68 = 2_000_000.;
    let result = state.max_hit_points(flags(0, 0, 0), &mut parts).unwrap();
    assert_eq!(result.hit_points, 32767);
    assert!(result.negative_short_replaced);
    for volume in [f32::INFINITY, f32::NAN, -1.] {
        state.cached_volume_68 = volume;
        let result = state.max_hit_points(flags(0, 0, 0), &mut parts).unwrap();
        assert_eq!(result.hit_points, 1);
        assert!(!result.negative_short_replaced);
    }
    let mut parts = Parts::new(vec![1.; 20_001]);
    let mut state = NativeDpHealthState::default();
    assert_eq!(
        state
            .max_hit_points(flags(0, 0, 0), &mut parts)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert!(parts.calls.is_empty());
    assert_eq!(state.cached_volume_68, 0.);
}

struct Animal {
    volume: f32,
    flying: Vec<bool>,
    calls: Vec<usize>,
    fail: Option<usize>,
}
impl NativeAnimaliaHealthContext for Animal {
    fn chromatid_volume(&mut self) -> io::Result<f32> {
        Ok(self.volume)
    }
    fn part_count(&mut self) -> io::Result<usize> {
        Ok(self.flying.len())
    }
    fn part_is_flying_wing(&mut self, index: usize) -> io::Result<bool> {
        self.calls.push(index);
        if self.fail == Some(index) {
            Err(io::ErrorKind::Unsupported.into())
        } else {
            Ok(self.flying[index])
        }
    }
}
#[test]
fn animalia_checks_actual_wings_and_stops_after_first_fly_capable_part() {
    let mut context = Animal {
        volume: 1.,
        flying: vec![false, true, true],
        calls: vec![],
        fail: None,
    };
    let result = native_animalia_max_hit_points(&mut context).unwrap();
    assert_eq!(result.hit_points, 19);
    assert_eq!(context.calls, [0, 1]);
    context.flying[1] = false;
    context.flying[2] = false;
    context.calls.clear();
    assert_eq!(
        native_animalia_max_hit_points(&mut context)
            .unwrap()
            .hit_points,
        39
    );
    assert_eq!(context.calls, [0, 1, 2]);
    context.calls.clear();
    context.fail = Some(1);
    assert_eq!(
        native_animalia_max_hit_points(&mut context)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(context.calls, [0, 1]);
}

#[test]
fn growth_helpers_preserve_full_byte_and_signed_short_arithmetic() {
    for byte in 0..=255 {
        assert_eq!(native_dp_growth_time(byte), byte - byte % 8 + 7);
    }
    for hp in i16::MIN..=i16::MAX {
        let expected = if hp < 0 {
            0
        } else {
            ((2_f64 * f64::from(hp).sqrt()).ceil() as u32).min(255) as u8
        };
        assert_eq!(native_animalia_growth_time(hp), expected, "growth HP{hp}");
    }
    for base in [i16::MIN, -30, 0, 1, 100, i16::MAX] {
        for growth in 0..=255 {
            for time in [0, 1, 7, 100, 255] {
                let result = native_growth_adjusted_hit_points(base, growth, time);
                if growth == 0 {
                    assert_eq!(result.unwrap(), base);
                } else if time == 0 {
                    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
                } else {
                    let raw = (i64::from(time) + 1 - i64::from(growth)) * i64::from(base)
                        / i64::from(time);
                    let short = raw as i16;
                    assert_eq!(result.unwrap(), if short == 0 { 1 } else { short });
                }
            }
        }
    }
}

#[test]
#[cfg(target_arch = "x86_64")]
fn host_sse_oracle_checks_conversion_limits_nan_and_low_short_diagnostics() {
    use std::arch::x86_64::{_mm_cvttss_si32, _mm_mul_ss, _mm_set_ss, _mm_sqrt_ss};
    let mut words = vec![
        0_u32,
        0x8000_0000,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
        0x4f00_0000,
        0xcf00_0000,
        0x4eff_ffff,
        0x3f80_0000,
    ];
    let mut random = 17_u32;
    for _ in 0..10000 {
        random = random.wrapping_mul(1664525).wrapping_add(1013904223);
        words.push(random);
    }
    for word in words {
        let value = f32::from_bits(word);
        // Host intrinsic execution is an independent arithmetic test, NOT
        // execution of the supplied executable or a native application oracle.
        assert_eq!(cvtt(value), unsafe { _mm_cvttss_si32(_mm_set_ss(value)) });
        for scale in [0x41ec_86eb, 0x421c_f478] {
            let raw = unsafe {
                _mm_cvttss_si32(_mm_mul_ss(
                    _mm_sqrt_ss(_mm_set_ss(value)),
                    _mm_set_ss(f32::from_bits(scale)),
                ))
            } as i16;
            let (hp, warn) = volume_hp(value, scale);
            assert_eq!(warn, raw < 0);
            assert_eq!(hp, if raw < 0 { 32767 } else { raw as u16 });
        }
    }
}
