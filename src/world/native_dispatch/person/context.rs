use super::*;
use crate::{
    protocol::QtReader,
    world::{
        native_dispatch::NativeAtomDispatchContext,
        native_person::{NativePersonCandidateReceiveContext, NativePersonOwnedReceiveContext},
        native_position_updates::NativePositionReceiver,
        native_prefix::NativeIdentityPrefix,
        native_signals::{NativeSignalConnector, connect_signals},
        native_subatoms::{NativeAtomOwnerContext, NativeAtomOwnerStores},
    },
};
use std::cell::RefCell;

impl<O: NativeAtomOwnerContext, S: NativeSignalConnector, H: NativePersonDispatchHooks>
    NativeAtomDispatchContext for NativePersonDispatch<'_, O, S, H>
{
    fn lookup(&self) -> &NativeRecoveryLookup {
        self.lookup
    }
    fn construct_candidate(&mut self, kind: NativeAtomKind) -> io::Result<u32> {
        if kind != NativeAtomKind::Person {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "concrete native dispatch class is not implemented",
            ));
        }
        let handle = self.hooks.allocate_handle(kind)?;
        self.people.construct_candidate(kind, handle, self.graph)?;
        self.hooks.construct_runtime(handle)?;
        Ok(handle)
    }
    fn assign_identity(&mut self, handle: u32, identity: NativeIdentityPrefix) -> io::Result<()> {
        self.people
            .get_mut(handle)
            .ok_or_else(|| io::Error::other("missing native Person candidate"))?
            .base
            .apply_received_identity(identity);
        Ok(())
    }
    fn assign_time(&mut self, handle: u32, time: i64) -> io::Result<()> {
        self.people
            .get_mut(handle)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "selected native class is not implemented in Person storage",
                )
            })?
            .base
            .received_time = time;
        Ok(())
    }
    fn receive_base(
        &mut self,
        handle: u32,
        _: NativeIdentityPrefix,
        input: &mut QtReader<'_>,
    ) -> io::Result<()> {
        let owned = self
            .people
            .get(handle)
            .ok_or_else(|| io::Error::other("missing selected native Person"))?
            .base
            .owner_attached;
        let now = self.hooks.position_epoch();
        // Hooks execute sequentially; temporary RefCell borrows do not escape
        // callbacks. This is Rust borrow coordination, not native Qt locking.
        let hooks = RefCell::new(&mut *self.hooks);
        if owned {
            let mut receiver = NativePositionReceiver {
                graph: self.graph,
                runaways: self.runaways,
                registrations: self.registrations,
                owner: self.owner,
            };
            self.people.receive_owned_base(
                handle,
                input,
                NativePersonOwnedReceiveContext {
                    receiver: &mut receiver,
                    lookup: self.lookup,
                    position_epoch: now,
                    profile: self.profile,
                },
                |record| hooks.borrow_mut().destination(handle, record),
                |handle| hooks.borrow_mut().position_auxiliary(handle),
                || hooks.borrow_mut().effect_clock(),
            )?;
        } else {
            self.people.receive_candidate_base(
                handle,
                input,
                NativePersonCandidateReceiveContext {
                    graph: self.graph,
                    lookup: self.lookup,
                    retired: self.retired,
                    position_epoch: now,
                    profile: self.profile,
                },
                |handle| hooks.borrow_mut().position_auxiliary(handle),
                || hooks.borrow_mut().effect_clock(),
            )?;
        }
        Ok(())
    }
    fn connect_signals(&mut self, handle: u32) -> io::Result<()> {
        connect_signals(handle, self.raw_mode, self.connector)?;
        Ok(())
    }
    fn add_atom(&mut self, handle: u32) -> io::Result<()> {
        self.people.add_candidate_owner(
            handle,
            self.graph,
            NativeAtomOwnerStores {
                lookup: self.lookup,
                runaways: self.runaways,
                registrations: self.registrations,
                retired: self.retired,
            },
            self.owner,
        )?;
        Ok(())
    }
    fn destroy_candidate(&mut self, handle: u32) -> io::Result<()> {
        let person = self
            .people
            .get(handle)
            .ok_or_else(|| io::Error::other("missing Person candidate"))?;
        self.graph.validate_unowned_candidate(
            handle,
            person.base.owner_attached,
            self.lookup,
            self.retired,
        )?;
        self.hooks.destroy_runtime(handle)?;
        self.people
            .discard_candidate(handle, self.graph, self.lookup, self.retired)
    }
    fn receive_body(&mut self, handle: u32, _: bool, input: &mut QtReader<'_>) -> io::Result<()> {
        let person = self.people.get_mut(handle).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "selected class is not implemented in Person storage",
            )
        })?;
        let report =
            person.receive_body(input, self.profile, self.hooks.person_context(handle)?)?;
        self.hooks.received_body(handle, report)
    }
    fn post_update(&mut self, handle: u32, is_new: bool) -> io::Result<()> {
        self.hooks.post_update(handle, is_new)
    }
}
