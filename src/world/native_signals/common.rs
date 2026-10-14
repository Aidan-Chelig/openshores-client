use super::NativeSignalConnection;

pub(super) const BINDINGS: &[NativeSignalConnection] = &[
    NativeSignalConnection {
        signal: "2OfficerKilled(AuId)",
        target: "1OfficerKilledUp(AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2AvatarOnLine(AuId, quint32)",
        target: "1AvatarOnLineUp(AuId, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2PostJobOffer(AuJobOffer)",
        target: "1PostJobOffer(AuJobOffer)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2WarpSignatureDetected(AuId, quint32, quint32)",
        target: "1WarpSignatureDetected(AuId, quint32, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SystemWarpSignature(AuId, quint32, quint8, float, float, float)",
        target: "1SystemWarpSignature(AuId, quint32, quint8, float, float, float)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2ReleaseBerthSignal(AuId, AuId)",
        target: "1ReleaseBerthUp(AuId, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SetPoliticalStance(quint32, quint32, quint8, quint8, AuId, QString, quint8, AuId, QString, quint8, quint8, QString, QString, QString)",
        target: "1SetPoliticalStance(quint32, quint32, quint8, quint8, AuId, QString, quint8, AuId, QString, quint8, quint8, QString, QString, QString)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SolarSystemExplored(AuId, quint32)",
        target: "1SolarSystemExplored(AuId, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SolarSystemScanned(AuId, quint32, quint8, quint32)",
        target: "1SolarSystemScanned(AuId, quint32, quint8, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SolarSystemFinal(AuId)",
        target: "1SolarSystemFinal(AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SolarSystemReadyToDecay(AuId)",
        target: "1SolarSystemReadyToDecay(AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2StargateClosed(AuId, AuPoint)",
        target: "1StargateClosed(AuId, AuPoint)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2StargateOpened(AuId, AuPoint, AuId, AuPoint, AuTime)",
        target: "1StargateOpened(AuId, AuPoint, AuId, AuPoint, AuTime)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2Spawn(AuId, quint32, AuId, AuId, AuId, quint8)",
        target: "1SpawnAtom(AuId, quint32, AuId, AuId, AuId, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TradeReadyToShip(AuId, AuTradeAgreement)",
        target: "1TradeReadyToShip(AuId, AuTradeAgreement)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TradeShipped(AuId, AuTradeAgreement)",
        target: "1TradeShipped(AuId, AuTradeAgreement)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2Travel(AuId, AuId, AuId, AuId)",
        target: "1AtomTraveled(AuId, AuId, AuId, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TravelGalaxyLoc(AuId, AuId, AuId, AuPoint)",
        target: "1AtomTraveledGalaxyLoc(AuId, AuId, AuId, AuPoint)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2TravelStargate(AuId, AuId, AuId, AuPoint)",
        target: "1AtomTraveledStargate(AuId, AuId, AuId, AuPoint)",
        raw_type: 2,
    },
];
