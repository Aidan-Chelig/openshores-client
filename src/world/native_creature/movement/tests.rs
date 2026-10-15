use super::*;
use std::collections::VecDeque;

struct Context {
    byte_29: u8,
    body: NativeMovementBody,
    growth: u8,
    time: u8,
    max_hp: VecDeque<i16>,
    burdens: VecDeque<f32>,
    events: Vec<&'static str>,
    fail: Option<&'static str>,
}
impl Context {
    fn new(body: NativeMovementBody) -> Self {
        Self {
            byte_29: 0,
            body,
            growth: 0,
            time: 100,
            max_hp: VecDeque::from([100, 100]),
            burdens: VecDeque::from([50., 50.]),
            events: vec![],
            fail: None,
        }
    }
    fn event(&mut self, name: &'static str) -> io::Result<()> {
        self.events.push(name);
        if self.fail == Some(name) {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        } else {
            Ok(())
        }
    }
}
impl NativeMovementContext for Context {
    fn dp_byte_29(&mut self) -> io::Result<u8> {
        self.event("dp-byte")?;
        Ok(self.byte_29)
    }
    fn body(&mut self) -> io::Result<NativeMovementBody> {
        self.event("body")?;
        Ok(self.body)
    }
    fn growth_byte_494(&mut self) -> io::Result<u8> {
        self.event("growth")?;
        Ok(self.growth)
    }
    fn growth_time(&mut self) -> io::Result<u8> {
        self.event("time")?;
        Ok(self.time)
    }
    fn max_hit_points(&mut self) -> io::Result<i16> {
        self.event("max-hp")?;
        Ok(self.max_hp.pop_front().expect("unexpected MaxHP sample"))
    }
    fn burden(&mut self) -> io::Result<f32> {
        self.event("burden")?;
        Ok(self.burdens.pop_front().expect("unexpected burden sample"))
    }
}

#[test]
fn ordered_observations_distinct_burden_samples_and_dp_high_nibble_capacity() {
    let mut context = Context::new(NativeMovementBody::DpBody { hit_points: 25 });
    context.byte_29 = 0x3f;
    context.burdens = VecDeque::from([1., 70.]); // capacity25*(100/10+1+3)=350
    assert_eq!(native_movement_speed_modifier(&mut context).unwrap(), 0.4);
    assert_eq!(
        context.events,
        [
            "body", "growth", "dp-byte", "max-hp", "burden", "burden", "max-hp"
        ]
    );
    for hp in [i16::MIN, -30, 0] {
        let mut context = Context::new(NativeMovementBody::Animalia { hit_points: hp });
        context.growth = 1;
        assert_eq!(native_movement_speed_modifier(&mut context).unwrap(), 1.);
        assert_eq!(context.events, ["body", "growth"]);
    }
    let mut context = Context::new(NativeMovementBody::Animalia { hit_points: 50 });
    context.burdens = VecDeque::from([200.]);
    assert_eq!(native_movement_speed_modifier(&mut context).unwrap(), 0.);
    assert_eq!(context.events, ["body", "growth", "burden"]);
    let mut context = Context::new(NativeMovementBody::Animalia { hit_points: 50 });
    context.burdens = VecDeque::from([1., 200.]);
    assert_eq!(native_movement_speed_modifier(&mut context).unwrap(), 0.);
    assert_eq!(context.events, ["body", "growth", "burden", "burden"]);
}

#[test]
fn growth_all_bytes_signed_short_truncation_and_native_zero_to_one() {
    for growth in 0..=255 {
        for time in [0, 1, 2, 100, 255] {
            for hp in [1_i16, 2, 16383, 16384, i16::MAX] {
                let mut context = Context::new(NativeMovementBody::Animalia { hit_points: hp });
                context.growth = growth;
                context.time = time;
                let actual = native_movement_speed_modifier(&mut context);
                if growth != 0 && time == 0 {
                    assert_eq!(actual.unwrap_err().kind(), io::ErrorKind::InvalidData);
                    continue;
                }
                let mut adjusted = if growth == 0 {
                    hp
                } else {
                    ((i32::from(time) + 1 - i32::from(growth)) * i32::from(hp) / i32::from(time))
                        as i16
                };
                if growth != 0 && adjusted == 0 {
                    adjusted = 1;
                }
                let expected = if adjusted <= 0 {
                    1.
                } else {
                    let doubled = i32::from(adjusted) * 2;
                    let wrapped = (doubled as u16) as i16;
                    if 100 > wrapped {
                        0.75 * (f32::from(wrapped) / 100.)
                    } else {
                        0.75
                    }
                };
                assert_eq!(
                    actual.unwrap().to_bits(),
                    expected.to_bits(),
                    "{growth}/{time}/{hp}"
                );
            }
        }
    }
}

#[test]
fn nonfinite_observations_are_not_silently_normalized_and_errors_stop_sampling() {
    for (first, second) in [
        (f32::NAN, 25.),
        (0., f32::NAN),
        (f32::INFINITY, 0.),
        (f32::NEG_INFINITY, f32::NEG_INFINITY),
        (-100., -100.),
    ] {
        let capacity = 200.;
        let mut context = Context::new(NativeMovementBody::Animalia { hit_points: 50 });
        context.burdens = VecDeque::from([first, second]);
        let expected = if first >= capacity {
            0.
        } else {
            let value = (capacity - second) / capacity;
            if value <= 0. { 0. } else { value }
        };
        let actual = native_movement_speed_modifier(&mut context).unwrap();
        if expected.is_nan() {
            assert!(actual.is_nan());
        } else {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }
    let expected = [
        "body", "growth", "dp-byte", "max-hp", "burden", "burden", "max-hp",
    ];
    for failure in ["body", "growth", "dp-byte", "max-hp", "burden"] {
        let mut context = Context::new(NativeMovementBody::DpBody { hit_points: 25 });
        context.fail = Some(failure);
        assert_eq!(
            native_movement_speed_modifier(&mut context)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        let end = expected.iter().position(|name| *name == failure).unwrap();
        assert_eq!(context.events, expected[..=end]);
    }
}
