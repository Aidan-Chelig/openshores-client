use super::{Context, RegistrationAction};

pub(super) fn evaluate<C: Context>(
    id: u32,
    context: &mut C,
    mut emit: impl FnMut(&mut C, RegistrationAction),
) {
    use RegistrationAction::*;
    if context.must_not_tick_tock() {
        emit(context, ReleaseTickTock { id });
        return;
    }
    let mut solar = None;
    if context.is_ticker() {
        solar = context.find_parent_solar_system();
        if let Some(system) = solar.filter(|system| context.is_ticking_contents(*system)) {
            let atom_type = context.atom_type();
            emit(
                context,
                UpdateOrAddTicker {
                    id,
                    atom_type,
                    solar_id: system.id,
                },
            );
        } else {
            emit(context, ReleaseTicker { id });
        }
    } else {
        emit(context, ReleaseTicker { id });
    }
    let raw_interval = context.is_tocker();
    if raw_interval == 0 {
        emit(context, ReleaseTocker { id });
        return;
    }
    // Retry a missing ticker parent; reuse a resolved parent even when its
    // contents were not ticking. Both entry points share this exact algorithm.
    let solar = solar.or_else(|| context.find_parent_solar_system());
    if let Some(system) = solar.filter(|system| context.solar_field_3a9(*system) == 0) {
        let atom_type = context.atom_type();
        emit(
            context,
            UpdateOrAddTocker {
                id,
                atom_type,
                solar_id: system.id,
                raw_interval,
            },
        );
    } else {
        emit(context, ReleaseTocker { id });
    }
}
