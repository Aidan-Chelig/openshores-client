use super::NativeBaseReceiveContext;
use crate::{
    atom_position::PackedPosition,
    protocol::QtReader,
    world::{
        native_atom::NativeBaseAtomInitialization, native_controller::NativeControllerList,
        native_effect::NativeEffectList,
    },
};
use std::io;

/// Additional received base fields. None means unobserved, not native zero.
/// Constructor time defaults and rendering/application remain separate.
#[derive(Clone, Debug, Default)]
pub struct NativeBaseReceiveFields {
    pub created_ms: Option<i64>,
    pub modified_ms: Option<i64>,
    pub in_motion: Option<bool>,
    pub effects: NativeEffectList,
    pub controllers: NativeControllerList,
}

/// Concrete base field/layer adapter. Position callback must finish native
/// ChangePos plus any conditional 0xa0 reset on the selected object. An
/// unresolved reparent must return an error, not silently reset status.
/// Clock callback supplies per-effect wall-clock and global adjustment samples.
pub struct NativeBaseReceiveAdapter<'a, P, C> {
    pub atom: &'a mut NativeBaseAtomInitialization,
    pub fields: &'a mut NativeBaseReceiveFields,
    pub position: P,
    pub clock: C,
}
impl<P, C> NativeBaseReceiveContext for NativeBaseReceiveAdapter<'_, P, C>
where
    P: FnMut(&mut NativeBaseAtomInitialization, PackedPosition) -> io::Result<()>,
    C: FnMut() -> (i64, i64),
{
    fn receive_position(&mut self, position: PackedPosition) -> io::Result<()> {
        (self.position)(self.atom, position)
    }
    fn reset_status(&mut self) -> io::Result<()> {
        self.atom.finish_received_position_stage();
        Ok(())
    }
    fn assign_created_time(&mut self, time: i64) -> io::Result<()> {
        self.fields.created_ms = Some(time);
        Ok(())
    }
    fn assign_modified_time(&mut self, time: i64) -> io::Result<()> {
        self.fields.modified_ms = Some(time);
        Ok(())
    }
    fn receive_effects(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        self.fields.effects.receive(input, &mut self.clock)?;
        Ok(())
    }
    fn assign_motion(&mut self, in_motion: bool) -> io::Result<()> {
        self.fields.in_motion = Some(in_motion);
        Ok(())
    }
    fn receive_controllers(&mut self, input: &mut QtReader<'_>) -> io::Result<()> {
        self.fields.controllers.receive_base_layer(input)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{atom_position::FloatEncoding, world::native_dispatch::base::receive_base};
    #[test]
    fn concrete_fields_receive_times_effects_controllers_and_sparse_omissions() {
        let mut atom = NativeBaseAtomInitialization::from_default_base_constructor();
        let mut fields = NativeBaseReceiveFields::default();
        let mut bytes = vec![0x76];
        bytes.extend((-11_i64).to_be_bytes());
        bytes.extend((-22_i64).to_be_bytes());
        bytes.extend([1, 0, 0, 7, 2, 2, 2, 0xde]);
        let mut calls = 0;
        let mut adapter = NativeBaseReceiveAdapter {
            atom: &mut atom,
            fields: &mut fields,
            position: |_: &mut NativeBaseAtomInitialization, _: PackedPosition| {
                panic!("omitted position")
            },
            clock: || {
                calls += 1;
                (0x10000, 0)
            },
        };
        let mut input = QtReader::new(&bytes);
        receive_base(&mut input, FloatEncoding::TypeWidths, &mut adapter).unwrap();
        assert_eq!(input.remaining_bytes(), &[0xde]);
        assert_eq!(adapter.atom.field_1c, 0);
        assert_eq!(adapter.fields.created_ms, Some(-11));
        assert_eq!(adapter.fields.modified_ms, Some(-22));
        assert_eq!(adapter.fields.effects.effects()[0].time_70, 0x10007);
        assert_eq!(adapter.fields.controllers.entries().len(), 2);
        receive_base(
            &mut QtReader::new(&[0]),
            FloatEncoding::TypeWidths,
            &mut adapter,
        )
        .unwrap();
        assert_eq!(adapter.fields.in_motion, Some(false));
        assert_eq!(adapter.fields.created_ms, Some(-11));
        assert_eq!(adapter.fields.controllers.entries().len(), 2);
        assert_eq!(calls, 1);
    }
    #[test]
    fn unresolved_position_cannot_advance_status_or_later_fields() {
        let mut atom = NativeBaseAtomInitialization::from_default_base_constructor();
        let mut fields = NativeBaseReceiveFields::default();
        let mut adapter = NativeBaseReceiveAdapter {
            atom: &mut atom,
            fields: &mut fields,
            position: |_: &mut NativeBaseAtomInitialization, _: PackedPosition| {
                Err(io::Error::other("unresolved owner"))
            },
            clock: || panic!("later stage"),
        };
        assert!(
            receive_base(
                &mut QtReader::new(&[0x71, 0]),
                FloatEncoding::TypeWidths,
                &mut adapter
            )
            .is_err()
        );
        assert_eq!(adapter.atom.field_1c, 5);
        assert_eq!(adapter.fields.in_motion, None);
        assert!(!adapter.fields.effects.allocated());
        assert!(!adapter.fields.controllers.allocated());
    }
}
