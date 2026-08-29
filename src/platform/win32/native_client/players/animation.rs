//! Animation catalog reads.

use super::*;

impl NativeClientProfile {
    /// Copies the selected profile's fixed animation catalog.
    pub(crate) fn animation_catalog(self) -> Result<Vec<AnimationSnapshot>, DirectClientError> {
        let layout = self.spec.players.animation;
        let length = layout
            .entry_count
            .get()
            .checked_mul(layout.entry_size.get())
            .ok_or(DirectClientError::NotReady)?;
        let table = self
            .module_base
            .checked_add(layout.rva.get())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(table as *const u8, length) {
            return Err(DirectClientError::NotReady);
        }
        let entries = unsafe { std::slice::from_raw_parts(table as *const u8, length) };
        entries
            .chunks_exact(layout.entry_size.get())
            .map(parse_animation_entry)
            .collect()
    }
}

fn parse_animation_entry(entry: &[u8]) -> Result<AnimationSnapshot, DirectClientError> {
    let length = entry
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(entry.len());
    let Some(separator) = entry[..length].iter().position(|byte| *byte == b':') else {
        return Err(DirectClientError::NotReady);
    };
    let (name, file) = (&entry[..separator], &entry[separator + 1..length]);
    if name.is_empty() || file.is_empty() || file.contains(&b':') {
        return Err(DirectClientError::NotReady);
    }
    Ok(AnimationSnapshot {
        name: name.to_vec(),
        file: file.to_vec(),
    })
}
