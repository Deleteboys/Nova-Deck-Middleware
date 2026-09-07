//! Speicher- und Handle-Zähler über die Win32-Process-Status-API.

use crate::platform::ProcessDiagnostics;
use std::mem::size_of;
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, GetProcessHandleCount,
};

pub fn process_snapshot() -> ProcessDiagnostics {
    unsafe {
        let process = GetCurrentProcess();
        let mut counters = PROCESS_MEMORY_COUNTERS_EX {
            cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            ..Default::default()
        };

        let _ = GetProcessMemoryInfo(
            process,
            &mut counters as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
            size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        );

        let mut handle_count = 0u32;
        let _ = GetProcessHandleCount(process, &mut handle_count);

        ProcessDiagnostics {
            pid: GetCurrentProcessId(),
            working_set_bytes: counters.WorkingSetSize as u64,
            peak_working_set_bytes: counters.PeakWorkingSetSize as u64,
            private_usage_bytes: counters.PrivateUsage as u64,
            peak_pagefile_usage_bytes: counters.PeakPagefileUsage as u64,
            pagefile_usage_bytes: counters.PagefileUsage as u64,
            paged_pool_bytes: counters.QuotaPagedPoolUsage as u64,
            nonpaged_pool_bytes: counters.QuotaNonPagedPoolUsage as u64,
            page_faults: counters.PageFaultCount as u64,
            handle_count,
        }
    }
}
