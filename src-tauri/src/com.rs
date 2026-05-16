use std::cell::Cell;
use windows::core::{Result, HRESULT};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED,
};

const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x80010106u32 as i32);

thread_local! {
    static COM_DEPTH: Cell<usize> = Cell::new(0);
    static COM_SHOULD_UNINITIALIZE: Cell<bool> = Cell::new(false);
}

pub struct ComGuard {
    active: bool,
}

impl ComGuard {
    pub unsafe fn init_multithreaded() -> Result<Self> {
        Self::init(COINIT_MULTITHREADED)
    }

    pub unsafe fn init_apartment_threaded() -> Result<Self> {
        Self::init(COINIT_APARTMENTTHREADED)
    }

    unsafe fn init(coinit: COINIT) -> Result<Self> {
        if COM_DEPTH.with(|depth| depth.get()) > 0 {
            COM_DEPTH.with(|depth| depth.set(depth.get() + 1));
            return Ok(Self { active: true });
        }

        let result = CoInitializeEx(None, coinit);

        if result.is_ok() {
            COM_DEPTH.with(|depth| depth.set(1));
            COM_SHOULD_UNINITIALIZE.with(|should| should.set(true));
            return Ok(Self { active: true });
        }

        if result == RPC_E_CHANGED_MODE {
            COM_DEPTH.with(|depth| depth.set(1));
            COM_SHOULD_UNINITIALIZE.with(|should| should.set(false));
            return Ok(Self { active: true });
        }

        result.ok()?;
        unreachable!()
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let should_uninitialize = COM_DEPTH.with(|depth| {
            let current = depth.get();
            if current > 1 {
                depth.set(current - 1);
                false
            } else {
                depth.set(0);
                COM_SHOULD_UNINITIALIZE.with(|should| {
                    let value = should.get();
                    should.set(false);
                    value
                })
            }
        });

        if should_uninitialize {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
