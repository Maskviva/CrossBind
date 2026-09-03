mod build;
mod table;
#[cfg(test)]
mod tests;

pub use build::*;
pub use table::*;

use std::sync::OnceLock;

static ACTIVE: OnceLock<ItemRemap> = OnceLock::new();

pub fn build_once(server: &[(String, i32)]) -> Option<(&'static ItemRemap, bool)> {
    let client = client_items()?;
    let mut built_here = false;
    if ACTIVE.get().is_none() {
        built_here = ACTIVE.set(ItemRemap::build(client, server)).is_ok();
    }
    ACTIVE.get().map(|remap| (remap, built_here))
}

pub fn active() -> Option<&'static ItemRemap> {
    ACTIVE.get()
}

pub fn to_client(id: i32) -> i32 {
    match ACTIVE.get() {
        Some(remap) => remap.to_client(id),
        None => id,
    }
}

pub fn to_server(id: i32) -> i32 {
    match ACTIVE.get() {
        Some(remap) => remap.to_server(id),
        None => id,
    }
}
