use crate::action::actions::Action;
use crate::audio::{default_device_id, list_audio_devices, set_default_device};
use log::{debug, error};

#[derive(Debug)]
pub struct SwitchAudioAction {
    pub device_a: String,
    pub device_b: String,
}

impl Action for SwitchAudioAction {
    fn execute(&self) {
        let device_a = self.device_a.clone();
        let device_b = self.device_b.clone();

        tauri::async_runtime::spawn(async move {
            let devices = match list_audio_devices() {
                Ok(devices) => devices,
                Err(e) => {
                    error!("Fehler beim Auslesen der Audiogeräte: {}", e);
                    return;
                }
            };

            let find = |name: &str| {
                devices
                    .iter()
                    .find(|device| device.name == name)
                    .map(|device| device.id.clone())
            };

            let (Some(id_a), Some(id_b)) = (find(&device_a), find(&device_b)) else {
                error!(
                    "Eines der Geräte (A: '{}' oder B: '{}') wurde nicht gefunden.",
                    device_a, device_b
                );
                return;
            };

            // Auf das jeweils andere Gerät wechseln
            let current = default_device_id().unwrap_or(None);
            let (target_id, target_name) = if current.as_deref() == Some(id_a.as_str()) {
                (id_b, device_b)
            } else {
                (id_a, device_a)
            };

            match set_default_device(&target_id) {
                Ok(()) => debug!("Audio erfolgreich auf {} umgeschaltet.", target_name),
                Err(e) => error!("Fehler beim Umschalten auf {}: {}", target_name, e),
            }
        });
    }
}
