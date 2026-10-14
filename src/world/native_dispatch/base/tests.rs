use super::*;
use crate::world::native_atom::NativeBaseAtomInitialization;

struct Context {
    atom: NativeBaseAtomInitialization,
    events: Vec<&'static str>,
    motion: Option<bool>,
}
impl NativeBaseReceiveContext for Context {
    fn receive_position(&mut self, position: PackedPosition) -> io::Result<()> {
        self.events.push("position");
        self.atom
            .position
            .apply_fresh_wire_position(&position, false)?;
        Ok(())
    }
    fn reset_status(&mut self) -> io::Result<()> {
        self.events.push("status");
        self.atom.finish_received_position_stage();
        Ok(())
    }
    fn assign_created_time(&mut self, time: i64) -> io::Result<()> {
        self.events.push("created");
        assert_eq!(time, -11);
        Ok(())
    }
    fn assign_modified_time(&mut self, time: i64) -> io::Result<()> {
        self.events.push("modified");
        assert_eq!(time, -22);
        Ok(())
    }
    fn receive_effects(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        self.events.push("effects");
        // Established zero count: no inner AuEffect format needed here.
        assert_eq!(input.read_u8()?, 0);
        Ok(())
    }
    fn assign_motion(&mut self, in_motion: bool) -> io::Result<()> {
        self.events.push("motion");
        self.motion = Some(in_motion);
        Ok(())
    }
    fn receive_controllers(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        self.events.push("controllers");
        // Synthetic callback boundary marker, NOT a native controller format.
        assert_eq!(input.read_u8()?, 0xcc);
        Ok(())
    }
}
fn context() -> Context {
    Context {
        atom: NativeBaseAtomInitialization::from_default_base_constructor(),
        events: vec![],
        motion: None,
    }
}
#[test]
fn all_flags_preserve_stage_order_and_body_boundary() {
    for flags in 0..=u8::MAX {
        let mut bytes = vec![flags];
        let mut expected = vec![];
        if flags & 1 != 0 {
            bytes.push(2);
            bytes.extend(12.5_f32.to_be_bytes());
            expected.push("position");
        }
        expected.push("status");
        if flags & 2 != 0 {
            bytes.extend((-11_i64).to_be_bytes());
            expected.push("created");
        }
        if flags & 4 != 0 {
            bytes.extend((-22_i64).to_be_bytes());
            expected.push("modified");
        }
        if flags & 0x10 != 0 {
            bytes.push(0);
            expected.push("effects");
        }
        expected.push("motion");
        if flags & 0x40 != 0 {
            bytes.push(0xcc);
            expected.push("controllers");
        }
        let base_length = bytes.len();
        bytes.extend([0xde, 0xad]);
        let mut input = QtReader::new(&bytes);
        let mut context = context();
        let outcome = receive_base(&mut input, FloatEncoding::TypeWidths, &mut context).unwrap();
        assert_eq!(
            outcome,
            NativeBaseReceiveOutcome {
                flags,
                consumed_bytes: base_length
            }
        );
        assert_eq!(context.events, expected);
        assert_eq!(context.atom.field_1c, 0);
        assert_eq!(context.motion, Some(flags & 0x20 != 0));
        assert_eq!(
            context.atom.position.position.local[3][0],
            if flags & 1 != 0 { 12.5 } else { 0.0 }
        );
        assert_eq!(input.remaining_bytes(), &[0xde, 0xad]);
    }
}
#[test]
fn later_failure_retains_status_position_and_completed_times() {
    let mut bytes = vec![7, 2];
    bytes.extend(12.5_f32.to_be_bytes());
    bytes.extend((-11_i64).to_be_bytes());
    bytes.push(0);
    let mut input = QtReader::new(&bytes);
    let mut context = context();
    assert!(receive_base(&mut input, FloatEncoding::TypeWidths, &mut context).is_err());
    assert_eq!(context.events, ["position", "status", "created"]);
    assert_eq!(context.atom.field_1c, 0);
    assert_eq!(context.atom.position.position.local[3][0], 12.5);
    assert_eq!(context.motion, None);
    assert_eq!(input.remaining_bytes(), &[0]);
}
#[test]
fn truncated_position_does_not_reset_status() {
    let mut input = QtReader::new(&[1, 2]);
    let mut context = context();
    assert!(receive_base(&mut input, FloatEncoding::TypeWidths, &mut context).is_err());
    assert!(context.events.is_empty());
    assert_eq!(context.atom.field_1c, 5);
    assert_eq!(input.remaining_bytes(), &[2]);
}
