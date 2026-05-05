use log::{debug, error, info};
use crate::action::actions::Action;
use windows::core::{Interface, GUID, HRESULT, PCWSTR, IUnknown};
use windows::Win32::Media::Audio::{
    eCommunications, eConsole, eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
    ERole,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};
use crate::audio::list_audio_devices;

#[repr(C)]
pub struct IPolicyConfigVtbl {
    pub query_interface: usize,
    pub add_ref: usize,
    pub release: usize,
    pub get_mix_format: usize,
    pub get_device_format: usize,
    pub set_device_format: usize,
    pub get_processing_period: usize,
    pub set_processing_period: usize,
    pub get_share_mode: usize,
    pub set_share_mode: usize,
    pub get_property_value: usize,
    pub set_property_value: usize,
    pub set_default_endpoint: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        pszdevicename: PCWSTR,
        role: ERole,
    ) -> HRESULT,
    pub set_endpoint_visibility: usize,
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
pub struct PolicyConfig(pub IUnknown);

unsafe impl Interface for PolicyConfig {
    type Vtable = IPolicyConfigVtbl;
    const IID: GUID = GUID::from_u128(0x568b9108_44bf_40b4_9006_86afe5b5a620);
}

#[derive(Debug)]
pub struct SwitchAudioAction {
    pub device_a: String,
    pub device_b: String,
}

impl Action for SwitchAudioAction {
    fn execute(&self) {
        unsafe {
            // COM Initialisierung für diesen Thread[cite: 2]
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            // 1. Alle verfügbaren Geräte über das Audio-Modul abrufen[cite: 2]
            let Ok(devices) = list_audio_devices() else {
                error!("❌ Fehler beim Auslesen der Audiogeräte.");
                return;
            };

            // 2. IDs für die konfigurierten Namen finden
            let id_a = devices.iter().find(|d| d.name == self.device_a).map(|d| d.id.clone());
            let id_b = devices.iter().find(|d| d.name == self.device_b).map(|d| d.id.clone());

            let (Some(target_a), Some(target_b)) = (id_a, id_b) else {
                error!("❌ Eines der Geräte (A: '{}' oder B: '{}') wurde nicht gefunden.", self.device_a, self.device_b);
                return;
            };

            // 3. Aktuelles Standard-Gerät ermitteln[cite: 2]
            let Ok(enumerator) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else { return };
            let Ok(current_endpoint) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else { return };
            let Ok(current_id) = current_endpoint.GetId() else { return };
            let current_id_str = current_id.to_string().unwrap_or_default();

            let target_id = if current_id_str == target_a { &target_b } else { &target_a };
            let target_name = if current_id_str == target_a { &self.device_b } else { &self.device_a };

            let id_wide: Vec<u16> = target_id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr_id = PCWSTR(id_wide.as_ptr());

            let clsids = [
                GUID::from_u128(0x294935ce_f637_4e7c_a41b_ab255460b862), // Modern (Win 10/11)
                GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9), // Standard (Win 7/10)
            ];

            let mut success = false;
            for clsid in &clsids {
                if let Ok(pc) = CoCreateInstance::<_, PolicyConfig>(clsid, None, CLSCTX_ALL) {
                    let _ = (pc.vtable().set_default_endpoint)(pc.as_raw(), pcwstr_id, eConsole);
                    let _ = (pc.vtable().set_default_endpoint)(pc.as_raw(), pcwstr_id, eMultimedia);
                    let _ = (pc.vtable().set_default_endpoint)(pc.as_raw(), pcwstr_id, eCommunications);
                    success = true;
                    break;
                }
            }

            if success {
                debug!("✅ Audio erfolgreich auf {} umgeschaltet.", target_name);
            } else {
                error!("❌ Fehler: PolicyConfig konnte nicht instanziiert werden.");
            }
        }
    }
}