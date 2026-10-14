use super::NativeSignalConnection;

pub(super) const BINDINGS: &[NativeSignalConnection] = &[
    NativeSignalConnection {
        signal: "2AddCitizenToEmpire(quint32, quint32, AuId, QString, quint8)",
        target: "1AddCitizenToEmpire(quint32, quint32, AuId, QString, quint8)",
        raw_type: 130,
    },
    NativeSignalConnection {
        signal: "2ArenaReady(AuArenaReady)",
        target: "1ArenaReady(AuArenaReady)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2AvatarAccountActive(AuIdList)",
        target: "1AvatarAccountActive(AuIdList)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2AvatarBorn(AuId, AuId)",
        target: "1AvatarBorn(AuId, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CityFacilitiesChanged(AuId, AuCityIndustryHash)",
        target: "1CityFacilitiesChanged(AuId, AuCityIndustryHash)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CityStatusChanged(AuId, int, qint8)",
        target: "1CityStatusChanged(AuId, int, qint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CityCreated(quint32, quint8, AuId, QString, AuId, QString, AuId, QString, AuId, QString, AuPoint, AuId, QString, AuId, quint8)",
        target: "1CityCreated(quint32, quint8, AuId, QString, AuId, QString, AuId, QString, AuId, QString, AuPoint, AuId, QString, AuId, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2EnteredDesigner(AuId, int)",
        target: "2EnteredDesigner(AuId, int)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2GovDossierAddDocument(quint32, quint32, QByteArray)",
        target: "1GovDossierAddDocument(quint32, quint32, QByteArray)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2GovSetCapital(quint32, AuId, AuSpecies, AuId, int)",
        target: "1GovSetCapitalUp(quint32, AuId, AuSpecies, AuId, int)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2DemiavatarDied(AuId)",
        target: "1DemiavatarDied(AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2HengeActivatedSignal(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        target: "1HengeActivated(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2HengeCreatedSignal(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        target: "1HengeCreated(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2HengeDeactivatedSignal(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        target: "1HengeDeactivated(AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2HengeDestinationSignal(AuId, AuId, QString, quint8, QString, QString, quint8, AuGlobeLLF, bool, quint8, bool)",
        target: "1HengeDestinationConfirmed(AuId, AuId, QString, quint8, QString, QString, quint8, AuGlobeLLF, bool, quint8, bool)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2HengeTeleportSignal(AuId, AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        target: "1HengeTeleporterEntered(AuId, AuId, quint8, quint8, quint8, AuId, quint8, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2OfficeAssigned(AuId, quint32, AuOffice)",
        target: "1OfficeAssigned(AuId, quint32, AuOffice)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2OpenChannelTo(AuId, QString, AuId, quint8, AuId, QString, quint8, bool)",
        target: "1OpenChannelTo(AuId, QString, AuId, quint8, AuId, QString, quint8, bool)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2EmpireRecordPatent(quint32, quint16, quint8)",
        target: "1EmpireRecordPatent(quint32, quint16, quint8)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2ClaimTitle(AuId, QString, AuId, QString, quint32)",
        target: "1ClaimTitleUp(AuId, QString, AuId, QString, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SectorPirateBaseEncounter(AuId, AuId)",
        target: "1SectorPirateBaseEncounter(AuId, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SendFunds(AuId, QString, AuId, double, quint8, quint8, quint32)",
        target: "1SendFundsUp(AuId, QString, AuId, double, quint8, quint8, quint32)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2ShipIt(AuItemTEUContainer)",
        target: "1ShipIt(AuItemTEUContainer)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SpacecraftNameChanged(AuId, QString)",
        target: "1SpacecraftNameChanged(AuId, QString)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2SpacecraftPositionReport(AuId, AuId, AuStarMapSpacecraft)",
        target: "1SpacecraftPositionReport(AuId, AuId, AuStarMapSpacecraft)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2BountyPosted(AuId, QString, AuId, QString, double, int)",
        target: "1BountyPosted(AuId, QString, AuId, QString, double, int)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CityCaptured(quint32, quint32, AuId)",
        target: "1CityCaptured(quint32, quint32, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2CityDecayed(quint32, AuId)",
        target: "1CityDecayed(quint32, AuId)",
        raw_type: 0,
    },
    NativeSignalConnection {
        signal: "2EmpireCityWorkerSpeciesChanged(quint32, AuId, AuSpecies)",
        target: "1EmpireCityWorkerSpeciesChanged(quint32, AuId, AuSpecies)",
        raw_type: 0,
    },
    NativeSignalConnection {
        signal: "2CreditCityBank(float, AuId)",
        target: "1CreditCityBank(float, AuId)",
        raw_type: 2,
    },
    NativeSignalConnection {
        signal: "2GameOver(AuId, quint16, quint8, QByteArray)",
        target: "2GameOver(AuId, quint16, quint8, QByteArray)",
        raw_type: 2,
    },
];
