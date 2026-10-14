use super::NativeSignalConnection;

pub(super) const BINDINGS: &[NativeSignalConnection] = &[
    NativeSignalConnection {
        signal: "2OperateTurboLift(AuId, AuId, quint16)",
        target: "1OperateTurboLiftUI(AuId, AuId, quint16)",
        raw_type: 130,
    },
    NativeSignalConnection {
        signal: "2EmpireRequested(quint32)",
        target: "1EmpireRequested(quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2RestoreConsoleSettings(quint8, QByteArray)",
        target: "1RestoreConsoleSettings(quint8, QByteArray)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2EnteredWormholeOnClient(AuId, AuId, AuWormhole, float)",
        target: "1AtomEnteredWormhole(AuId, AuId, AuWormhole, float)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CollisionOnClient(AuId, AuPoint, AuPoint, AuId, AuPoint, AuPoint, AuPoint, bool)",
        target: "1AtomCollision(AuId, AuPoint, AuPoint, AuId, AuPoint, AuPoint, AuPoint, bool)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2DeadheadedOnClient(AuId, AuId, AuPoint)",
        target: "1AtomDeadheaded(AuId, AuId, AuPoint)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2FellOnClient(AuId, quint32, int)",
        target: "1AtomFell(AuId, quint32, int)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TransporterCoord(AuPoint)",
        target: "1TransporterCoordUI(AuPoint)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TransporterLongLat(AuGlobeLLF)",
        target: "1TransporterLongLatUI(AuGlobeLLF)",
        raw_type: 2,
    },
];
