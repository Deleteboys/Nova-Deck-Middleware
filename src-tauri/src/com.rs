use windows::core::{Result, HRESULT};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
};

const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x80010106u32 as i32);

pub struct ComGuard {
    should_uninitialize: bool,
}

impl ComGuard {
    pub unsafe fn init_multithreaded() -> Result<Self> {
        Self::init(COINIT_MULTITHREADED)
    }

    pub unsafe fn init_apartment_threaded() -> Result<Self> {
        Self::init(COINIT_APARTMENTTHREADED)
    }

    unsafe fn init(coinit: COINIT) -> Result<Self> {
        let result = CoInitializeEx(None, coinit);

        if result.is_ok() {
            return Ok(Self {
                should_uninitialize: true,
            });
        }

        if result == RPC_E_CHANGED_MODE {
            return Ok(Self {
                should_uninitialize: false,
            });
        }

        result.ok()?;
        unreachable!()
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.should_uninitialize {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
