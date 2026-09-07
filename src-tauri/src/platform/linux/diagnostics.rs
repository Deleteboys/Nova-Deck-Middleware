//! Speicher- und Deskriptor-Zähler aus `/proc/self`.
//!
//! Die Felder sind auf die Windows-Entsprechungen abgebildet, damit die
//! DevTools-Ansicht dieselbe Struktur bekommt. Für die kernelseitigen
//! Pool-Zähler gibt es unter Linux kein Gegenstück; sie bleiben 0.

use crate::platform::ProcessDiagnostics;
use std::fs;

pub fn process_snapshot() -> ProcessDiagnostics {
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();

    ProcessDiagnostics {
        pid: std::process::id(),
        working_set_bytes: status_bytes(&status, "VmRSS"),
        peak_working_set_bytes: status_bytes(&status, "VmHWM"),
        // Anonymer RSS ist der nicht mit anderen Prozessen geteilte Anteil und
        // damit das nächste Gegenstück zu "Private Usage" unter Windows.
        private_usage_bytes: status_bytes(&status, "RssAnon"),
        peak_pagefile_usage_bytes: status_bytes(&status, "VmPeak"),
        pagefile_usage_bytes: status_bytes(&status, "VmSize"),
        // Paged/Nonpaged Pool sind Kontingente des Windows-Kernels.
        paged_pool_bytes: 0,
        nonpaged_pool_bytes: 0,
        page_faults: page_faults(),
        handle_count: open_descriptors(),
    }
}

/// Liest eine Zeile der Form `VmRSS:      1234 kB` aus `/proc/self/status`
/// und rechnet sie in Bytes um.
fn status_bytes(status: &str, key: &str) -> u64 {
    status
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(':'))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|kilobytes| kilobytes * 1024)
        .unwrap_or(0)
}

/// Minor- und Major-Faults aus `/proc/self/stat`.
///
/// Der `comm`-Eintrag kann Leerzeichen und Klammern enthalten, deshalb wird ab
/// der letzten schließenden Klammer gezählt. Danach ist Feld 3 (`state`) der
/// erste Eintrag, `minflt` (Feld 10) also Index 7 und `majflt` (Feld 12) Index 9.
fn page_faults() -> u64 {
    let stat = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let Some((_, tail)) = stat.rsplit_once(')') else {
        return 0;
    };

    let fields: Vec<&str> = tail.split_whitespace().collect();
    let field = |index: usize| {
        fields
            .get(index)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0)
    };

    field(7) + field(9)
}

/// Offene Dateideskriptoren – das Linux-Pendant zu den Windows-Handles.
fn open_descriptors() -> u32 {
    fs::read_dir("/proc/self/fd")
        .map(|entries| entries.count() as u32)
        .unwrap_or(0)
}
