//! Umschalten des Standard-Wiedergabegeräts über das undokumentierte
//! `IPolicyConfig`-Interface – Windows bietet dafür keine offizielle API.

use crate::platform::windows::com::ComGuard;
use crate::platform::{PlatformError, Result};
use windows::core::{IUnknown, Interface, GUID, HRESULT, PCWSTR};
use windows::Win32::Media::Audio::{eCommunications, eConsole, eMultimedia, ERole};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

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

/// CLSIDs der PolicyConfig-Implementierung – je nach Windows-Version ist nur
/// eine der beiden vorhanden.
const POLICY_CONFIG_CLSIDS: [GUID; 2] = [
    GUID::from_u128(0x294935ce_f637_4e7c_a41b_ab255460b862), // Windows 10/11
    GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9), // Windows 7/10
];

pub fn set_default_endpoint(id: &str) -> Result<()> {
    unsafe {
        let _com = ComGuard::init_multithreaded()?;

        let id_wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        let pcwstr_id = PCWSTR(id_wide.as_ptr());

        for clsid in &POLICY_CONFIG_CLSIDS {
            let Ok(policy) = CoCreateInstance::<_, PolicyConfig>(clsid, None, CLSCTX_ALL) else {
                continue;
            };

            let set_default = policy.vtable().set_default_endpoint;
            let console = set_default(policy.as_raw(), pcwstr_id, eConsole);
            if console.is_err() {
                continue;
            }

            // Die weiteren Rollen dürfen fehlschlagen, ohne dass der Wechsel misslingt.
            let _ = set_default(policy.as_raw(), pcwstr_id, eMultimedia);
            let _ = set_default(policy.as_raw(), pcwstr_id, eCommunications);
            return Ok(());
        }

        Err(PlatformError::new(
            "PolicyConfig konnte nicht instanziiert werden",
        ))
    }
}
