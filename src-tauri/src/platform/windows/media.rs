//! Play/Pause pro Anwendung über die Global System Media Transport Controls.

use crate::audio::normalize_process_name;
use crate::platform::Result;
use log::debug;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

/// Schaltet Play/Pause bei allen Medien-Sessions, deren App-ID den Prozessnamen
/// enthält. Gibt die App-IDs der umgeschalteten Sessions zurück.
///
/// `join()` wartet blockierend auf die WinRT-Operationen; der Aufrufer bringt
/// dafür einen eigenen Thread mit.
pub fn toggle_play_pause(process_name: &str) -> Result<Vec<String>> {
    let needle = normalize_process_name(process_name);
    if needle.is_empty() {
        return Ok(Vec::new());
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.join()?;
    let sessions = manager.GetSessions()?;

    // Erst alle Operationen anstoßen, dann abwarten – so laufen sie parallel.
    let mut pending = Vec::new();
    for session in sessions {
        let app_id = session.SourceAppUserModelId()?.to_string();
        debug!("Medien-Session gefunden: {}", app_id);
        if app_id.to_lowercase().contains(&needle) {
            pending.push((app_id, session.TryTogglePlayPauseAsync()?));
        }
    }

    let mut toggled = Vec::with_capacity(pending.len());
    for (app_id, operation) in pending {
        operation.join()?;
        toggled.push(app_id);
    }

    Ok(toggled)
}
