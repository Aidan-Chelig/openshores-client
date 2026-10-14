//! Established ConnectSignals plans. Does not emulate Qt or execute handlers.
mod common;
mod mode10;
mod mode5;
mod other;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSignalConnection {
    /// Exact Qt signature, including signal/slot prefix and argument types.
    pub signal: &'static str,
    pub target: &'static str,
    pub raw_type: u32,
}
pub fn connection_plan(raw_mode: i32) -> Vec<NativeSignalConnection> {
    if raw_mode == 9 {
        return vec![];
    }
    let mut plan = match raw_mode {
        10 => mode10::BINDINGS.to_vec(),
        5 => mode5::BINDINGS.to_vec(),
        _ => other::BINDINGS.to_vec(),
    };
    if raw_mode == 5 {
        plan.push(NativeSignalConnection {
            signal: "2CreateEmpire()",
            target: "1CreateEmpireUI()",
            raw_type: 2,
        });
        return plan;
    }
    plan.push(NativeSignalConnection {
        signal: "2GameTurnPoll(AuId, quint16, AuId)",
        target: if raw_mode == 10 {
            "1GameTurn(AuId, quint16, AuId)"
        } else {
            "2GameTurn(AuId, quint16, AuId)"
        },
        raw_type: 2,
    });
    plan.extend_from_slice(common::BINDINGS);
    plan.push(NativeSignalConnection {
        signal: "2WormholeExplored(AuId, AuId, qint8, quint32, QString)",
        target: "1WormholeExplored(AuId, AuId, qint8, quint32, QString)",
        raw_type: 2,
    });
    plan
}

/// Application-singleton connector. Returning false is a connection rejection;
/// Rust errors stop with earlier connector effects retained. No default no-op.
pub trait NativeSignalConnector {
    fn connect(&mut self, atom_handle: u32, binding: NativeSignalConnection) -> io::Result<bool>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSignalInstall {
    pub attempted: usize,
    pub accepted: usize,
    pub first_rejected: bool,
}
pub fn connect_signals(
    handle: u32,
    raw_mode: i32,
    connector: &mut impl NativeSignalConnector,
) -> io::Result<NativeSignalInstall> {
    let mut outcome = NativeSignalInstall {
        attempted: 0,
        accepted: 0,
        first_rejected: false,
    };
    for binding in connection_plan(raw_mode) {
        let accepted = connector.connect(handle, binding)?;
        outcome.attempted += 1;
        outcome.accepted += usize::from(accepted);
        if outcome.attempted == 1 && !accepted {
            outcome.first_rejected = true;
            break;
        }
    }
    Ok(outcome)
}

#[cfg(test)]
mod tests;
