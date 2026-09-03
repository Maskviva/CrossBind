mod build;
mod table;
#[cfg(test)]
mod tests;

pub use build::*;
pub use table::*;

use std::sync::OnceLock;

#[cfg(not(test))]
static ACTIVE: OnceLock<ItemRemap> = OnceLock::new();

#[cfg(not(test))]
fn slot_set(build: impl FnOnce() -> ItemRemap) -> bool {
    if ACTIVE.get().is_none() {
        return ACTIVE.set(build()).is_ok();
    }
    false
}

#[cfg(not(test))]
fn slot_get() -> Option<&'static ItemRemap> {
    ACTIVE.get()
}

#[cfg(test)]
thread_local! {
    static ACTIVE: OnceLock<&'static ItemRemap> = const { OnceLock::new() };
}

#[cfg(test)]
fn slot_set(build: impl FnOnce() -> ItemRemap) -> bool {
    ACTIVE.with(|slot| {
        if slot.get().is_some() {
            return false;
        }
        slot.set(Box::leak(Box::new(build()))).is_ok()
    })
}

#[cfg(test)]
fn slot_get() -> Option<&'static ItemRemap> {
    ACTIVE.with(|slot| slot.get().copied())
}

pub fn build_once(server: &[(String, i32)]) -> Option<(&'static ItemRemap, bool)> {
    let client = client_items()?;
    let built_here = slot_set(|| ItemRemap::build(client, server));
    slot_get().map(|remap| (remap, built_here))
}

pub fn active() -> Option<&'static ItemRemap> {
    slot_get()
}

pub fn to_client(id: i32) -> i32 {
    match slot_get() {
        Some(remap) => remap.to_client(id),
        None => id,
    }
}

pub fn to_server(id: i32) -> i32 {
    match slot_get() {
        Some(remap) => remap.to_server(id),
        None => id,
    }
}
