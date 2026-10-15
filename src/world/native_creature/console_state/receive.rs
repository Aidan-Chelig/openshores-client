use super::*;

impl NativeConsoleState {
    /// Invoke constructor-confirmed primary+c0 station reader. Call only when
    /// the enclosing native matching/temporary/context logic requires a body.
    /// This method does not resolve that context or perform UI/settings effects.
    pub fn receive_wire_qt7(
        &mut self,
        input: &mut QtReader<'_>,
        profile: FloatEncoding,
    ) -> io::Result<NativeConsoleWireReceive> {
        let length = input.remaining();
        let sensor_stage = match &mut self.body {
            NativeConsoleBody::Empty { .. } | NativeConsoleBody::Command => None,
            NativeConsoleBody::Engineer(value) => {
                *value = NativeStationEngineer::read(input)?;
                None
            }
            NativeConsoleBody::Helm(value) => {
                *value = NativeStationHelm::read(input)?;
                None
            }
            NativeConsoleBody::Game(value) => {
                *value = NativeStationGame::read(input)?;
                None
            }
            NativeConsoleBody::Shield(value) => {
                *value = NativeStationShield::read(input, profile)?;
                None
            }
            NativeConsoleBody::Surgery(value) => {
                *value = NativeStationSurgery::read_qt7(input, profile)?;
                None
            }
            NativeConsoleBody::Nav(value) => {
                *value = NativeStationNav::read(input, profile)?;
                None
            }
            NativeConsoleBody::Power(value) => {
                value.receive(input)?;
                None
            }
            NativeConsoleBody::Sensor(value) => Some(value.receive_wire(input, profile)?),
            NativeConsoleBody::Weapon(value) => Some(value.receive_wire(input, profile)?),
            NativeConsoleBody::Transporter(value) => Some(value.receive_wire(input, profile)?),
        };
        Ok(NativeConsoleWireReceive {
            consumed_bytes: length - input.remaining(),
            sensor_stage,
        })
    }
}
