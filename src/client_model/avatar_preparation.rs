//! Bounded CPU appearance preparation. Ownership is not a geometry cache key.
use crate::{
    asset_mount::AssetMount,
    world::{
        native_body::{
            NativeDpAvatarGeometry, NativeDpPosedAvatarSurface, NativeDpPreparedAvatarSurface,
        },
        native_dna::NativeDhDna,
    },
};
use std::{
    io,
    sync::{
        Arc,
        mpsc::{Receiver, TryRecvError},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarContentKey {
    pub dna: NativeDhDna,
    pub asset_revision: u64,
    pub repair_seams: bool,
    angle_bits: u64,
}
impl AvatarContentKey {
    pub fn new(
        dna: NativeDhDna,
        asset_revision: u64,
        angle: f64,
        repair_seams: bool,
    ) -> io::Result<Self> {
        if !angle.is_finite() || !(0.0..=180.).contains(&angle) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "avatar smoothing angle must be finite and in 0..=180",
            ));
        }
        Ok(Self {
            dna,
            asset_revision,
            repair_seams,
            angle_bits: if angle == 0. {
                0_f64.to_bits()
            } else {
                angle.to_bits()
            },
        })
    }
    pub fn angle(&self) -> f64 {
        f64::from_bits(self.angle_bits)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarRequest {
    pub session_generation: u64,
    pub entity_id: u32,
    pub content: AvatarContentKey,
}

/// Normal skin only. Armor/helmet fields are not fabricated from source gear.
pub struct PreparedNormalAvatar {
    pub native: NativeDpAvatarGeometry,
    pub surfaces: Vec<NativeDpPosedAvatarSurface>,
    pub animation_surfaces: Vec<NativeDpPreparedAvatarSurface>,
    pub atlas_png: Vec<u8>,
}
impl PreparedNormalAvatar {
    pub fn generate(mount: &AssetMount, key: &AvatarContentKey) -> io::Result<Self> {
        let native = NativeDpAvatarGeometry::generate(mount, &key.dna, 0, 0)?;
        let animation_surfaces = if key.repair_seams {
            native.prepare_surfaces_with_attachment_repair(key.angle())?
        } else {
            native.prepare_surfaces(key.angle())?
        };
        let surfaces = animation_surfaces
            .iter()
            .map(|surface| surface.skin(native.body.bones()))
            .collect::<io::Result<_>>()?;
        let atlas_png = native.skin.read_skin(mount)?;
        Ok(Self {
            native,
            surfaces,
            animation_surfaces,
            atlas_png,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AvatarPreparationStatus<'a> {
    Inactive,
    Queued,
    Running,
    Ready,
    Failed(&'a str),
}

type JobResult<T> = Result<T, String>;
struct Job<T> {
    request: AvatarRequest,
    revision: u64,
    receiver: Receiver<JobResult<T>>,
}

/// At most one running job and one completed CPU cache entry. A stale job must
/// drain before another starts; its result is never published or cached.
pub struct AvatarPreparation<T> {
    desired: Option<AvatarRequest>,
    pending: Option<Job<T>>,
    cached: Option<(AvatarContentKey, Arc<T>)>,
    failure: Option<(AvatarRequest, String)>,
    revision: u64,
}
impl<T> Default for AvatarPreparation<T> {
    fn default() -> Self {
        Self {
            desired: None,
            pending: None,
            cached: None,
            failure: None,
            revision: 0,
        }
    }
}
impl<T> AvatarPreparation<T> {
    pub fn select(&mut self, request: Option<AvatarRequest>) -> bool {
        if self.desired == request {
            return false;
        }
        self.revision = self.revision.wrapping_add(1);
        self.desired = request;
        self.failure = None;
        true
    }
    pub fn ready(&self) -> Option<(&AvatarRequest, &Arc<T>)> {
        let request = self.desired.as_ref()?;
        let (key, value) = self.cached.as_ref()?;
        (key == &request.content).then_some((request, value))
    }
    pub fn status(&self) -> AvatarPreparationStatus<'_> {
        let Some(request) = &self.desired else {
            return AvatarPreparationStatus::Inactive;
        };
        if self.ready().is_some() {
            return AvatarPreparationStatus::Ready;
        }
        if let Some((failed, error)) = &self.failure
            && failed == request
        {
            return AvatarPreparationStatus::Failed(error);
        }
        if let Some(job) = &self.pending
            && job.request == *request
        {
            return AvatarPreparationStatus::Running;
        }
        AvatarPreparationStatus::Queued
    }
    pub fn start_next(
        &mut self,
        spawn: impl FnOnce(&AvatarRequest) -> Receiver<JobResult<T>>,
    ) -> bool {
        if self.pending.is_some() || self.status() != AvatarPreparationStatus::Queued {
            return false;
        }
        let request = self.desired.as_ref().unwrap().clone();
        let receiver = spawn(&request);
        self.pending = Some(Job {
            request,
            revision: self.revision,
            receiver,
        });
        true
    }
    pub fn poll(&mut self) -> bool {
        let Some(job) = &self.pending else {
            return false;
        };
        let result = match job.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => Err("avatar preparation worker stopped".into()),
        };
        let job = self.pending.take().unwrap();
        if self.revision != job.revision || self.desired.as_ref() != Some(&job.request) {
            return true;
        }
        match result {
            Ok(value) => {
                self.cached = Some((job.request.content, Arc::new(value)));
                self.failure = None;
            }
            Err(error) => self.failure = Some((job.request, error)),
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    fn request(generation: u64, entity_id: u32, revision: u64, dna: u8) -> AvatarRequest {
        AvatarRequest {
            session_generation: generation,
            entity_id,
            content: AvatarContentKey::new(NativeDhDna([dna; 24]), revision, 60., true).unwrap(),
        }
    }
    #[test]
    fn same_appearance_reuses_ready_data_across_movement_and_completed_sessions() {
        let mut state = AvatarPreparation::<u32>::default();
        let first = request(1, 42, 1, 1);
        state.select(Some(first.clone()));
        let (tx, rx) = channel();
        assert!(state.start_next(|_| rx));
        tx.send(Ok(7)).unwrap();
        state.poll();
        assert!(!state.select(Some(first)));
        assert!(!state.start_next(|_| panic!("movement must not rebuild")));
        state.select(None);
        assert!(state.ready().is_none());
        let next = request(2, 42, 1, 1);
        state.select(Some(next.clone()));
        assert_eq!(state.ready().unwrap().0, &next);
        assert_eq!(**state.ready().unwrap().1, 7);
        assert!(!state.start_next(|_| panic!("completed matching geometry is reusable")));
    }
    #[test]
    fn stale_reconnect_and_asset_jobs_never_attach_to_new_ownership() {
        for next in [
            request(2, 42, 1, 1),
            request(1, 43, 1, 1),
            request(1, 42, 2, 1),
            request(1, 42, 1, 2),
        ] {
            let mut state = AvatarPreparation::<u32>::default();
            state.select(Some(request(1, 42, 1, 1)));
            let (tx, rx) = channel();
            state.start_next(|_| rx);
            state.select(Some(next.clone()));
            assert!(!state.start_next(|_| panic!("only one job may run")));
            tx.send(Ok(9)).unwrap();
            assert!(state.poll());
            assert!(state.ready().is_none());
            let (tx, rx) = channel();
            assert!(state.start_next(|seen| {
                assert_eq!(seen, &next);
                rx
            }));
            tx.send(Ok(10)).unwrap();
            state.poll();
            assert_eq!(**state.ready().unwrap().1, 10);
        }
    }
    #[test]
    fn failures_are_sticky_and_disconnect_discards_pending_completion() {
        let mut state = AvatarPreparation::<u32>::default();
        state.select(Some(request(1, 42, 1, 1)));
        let (tx, rx) = channel();
        state.start_next(|_| rx);
        drop(tx);
        state.poll();
        assert!(matches!(state.status(), AvatarPreparationStatus::Failed(_)));
        assert!(!state.start_next(|_| panic!("do not retry every frame")));
        state.select(Some(request(2, 42, 1, 1)));
        let (tx, rx) = channel();
        state.start_next(|_| rx);
        state.select(None);
        tx.send(Ok(4)).unwrap();
        state.poll();
        assert_eq!(state.status(), AvatarPreparationStatus::Inactive);
        assert!(state.ready().is_none());
    }
    #[test]
    fn appearance_settings_are_validated_and_invalidate_geometry() {
        let key = AvatarContentKey::new(NativeDhDna([0; 24]), 1, 60., true).unwrap();
        for angle in [f64::NAN, f64::INFINITY, -1., 181.] {
            assert!(AvatarContentKey::new(key.dna.clone(), 1, angle, true).is_err());
        }
        assert_ne!(
            key,
            AvatarContentKey::new(key.dna.clone(), 1, 0., true).unwrap()
        );
        assert_ne!(
            key,
            AvatarContentKey::new(key.dna.clone(), 1, 60., false).unwrap()
        );
        assert_eq!(
            AvatarContentKey::new(key.dna.clone(), 1, 0., true).unwrap(),
            AvatarContentKey::new(key.dna, 1, -0., true).unwrap()
        );
    }
    #[test]
    fn removing_and_reselecting_the_same_request_invalidates_in_flight_work() {
        let mut state = AvatarPreparation::<u32>::default();
        let req = request(1, 42, 1, 1);
        state.select(Some(req.clone()));
        let (tx, rx) = channel();
        state.start_next(|_| rx);
        state.select(None);
        state.select(Some(req));
        tx.send(Ok(7)).unwrap();
        state.poll();
        assert!(state.ready().is_none());
        assert_eq!(state.status(), AvatarPreparationStatus::Queued);
    }
}
