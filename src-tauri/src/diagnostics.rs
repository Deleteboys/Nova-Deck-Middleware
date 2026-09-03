use serde::Serialize;
use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static STARTED_AT: OnceLock<Instant> = OnceLock::new();

static COM_INIT_CALLS: AtomicU64 = AtomicU64::new(0);
static COM_REAL_INITS: AtomicU64 = AtomicU64::new(0);
static COM_REUSED_INITS: AtomicU64 = AtomicU64::new(0);
static COM_CHANGED_MODE: AtomicU64 = AtomicU64::new(0);
static COM_UNINITS: AtomicU64 = AtomicU64::new(0);

static SERIAL_BYTES_READ: AtomicU64 = AtomicU64::new(0);
static SERIAL_MESSAGES: AtomicU64 = AtomicU64::new(0);
static SERIAL_PARSE_ERRORS: AtomicU64 = AtomicU64::new(0);
static SERIAL_BUFFER_DROPS: AtomicU64 = AtomicU64::new(0);
static SERIAL_ACCUMULATOR_LEN: AtomicU64 = AtomicU64::new(0);
static SERIAL_ACCUMULATOR_CAP: AtomicU64 = AtomicU64::new(0);
static SERIAL_ACCUMULATOR_MAX_LEN: AtomicU64 = AtomicU64::new(0);
static SERIAL_CONNECTS: AtomicU64 = AtomicU64::new(0);
static SERIAL_DISCONNECTS: AtomicU64 = AtomicU64::new(0);
static SERIAL_PICO_EVENTS_EMITTED: AtomicU64 = AtomicU64::new(0);
static SERIAL_HOST_COMMANDS_WRITTEN: AtomicU64 = AtomicU64::new(0);

static AUDIO_TICKS: AtomicU64 = AtomicU64::new(0);
static AUDIO_SNAPSHOTS: AtomicU64 = AtomicU64::new(0);
static AUDIO_SESSIONS_ENUMERATED: AtomicU64 = AtomicU64::new(0);
static AUDIO_SLOT_POLLS: AtomicU64 = AtomicU64::new(0);
static AUDIO_STATUS_ERRORS: AtomicU64 = AtomicU64::new(0);
static AUDIO_EMPTY_RESULTS: AtomicU64 = AtomicU64::new(0);
static AUDIO_UPDATES_EMITTED: AtomicU64 = AtomicU64::new(0);
static AUDIO_PICO_COMMANDS_SENT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
pub struct RuntimeDiagnostics {
    pub uptime_secs: u64,
    pub process: ProcessDiagnostics,
    pub serial: SerialDiagnostics,
    pub audio: AudioDiagnostics,
    pub com: ComDiagnostics,
}

#[derive(Serialize)]
pub struct ProcessDiagnostics {
    pub pid: u32,
    pub working_set_bytes: u64,
    pub peak_working_set_bytes: u64,
    pub private_usage_bytes: u64,
    pub peak_pagefile_usage_bytes: u64,
    pub pagefile_usage_bytes: u64,
    pub paged_pool_bytes: u64,
    pub nonpaged_pool_bytes: u64,
    pub page_faults: u64,
    pub handle_count: u32,
}

#[derive(Serialize)]
pub struct SerialDiagnostics {
    pub bytes_read: u64,
    pub messages_decoded: u64,
    pub parse_errors: u64,
    pub buffer_drops: u64,
    pub accumulator_len: u64,
    pub accumulator_capacity: u64,
    pub accumulator_max_len: u64,
    pub connects: u64,
    pub disconnects: u64,
    pub pico_events_emitted: u64,
    pub host_commands_written: u64,
}

#[derive(Serialize)]
pub struct AudioDiagnostics {
    pub ticks: u64,
    pub snapshots: u64,
    pub sessions_enumerated: u64,
    pub slot_polls: u64,
    pub status_errors: u64,
    pub empty_results: u64,
    pub updates_emitted: u64,
    pub pico_commands_sent: u64,
}

#[derive(Serialize)]
pub struct ComDiagnostics {
    pub init_calls: u64,
    pub real_inits: u64,
    pub reused_inits: u64,
    pub changed_mode_results: u64,
    pub uninits: u64,
}

pub fn init() {
    STARTED_AT.get_or_init(Instant::now);
}

pub fn snapshot() -> RuntimeDiagnostics {
    let started_at = STARTED_AT.get_or_init(Instant::now);

    RuntimeDiagnostics {
        uptime_secs: started_at.elapsed().as_secs(),
        process: process_snapshot(),
        serial: SerialDiagnostics {
            bytes_read: SERIAL_BYTES_READ.load(Ordering::Relaxed),
            messages_decoded: SERIAL_MESSAGES.load(Ordering::Relaxed),
            parse_errors: SERIAL_PARSE_ERRORS.load(Ordering::Relaxed),
            buffer_drops: SERIAL_BUFFER_DROPS.load(Ordering::Relaxed),
            accumulator_len: SERIAL_ACCUMULATOR_LEN.load(Ordering::Relaxed),
            accumulator_capacity: SERIAL_ACCUMULATOR_CAP.load(Ordering::Relaxed),
            accumulator_max_len: SERIAL_ACCUMULATOR_MAX_LEN.load(Ordering::Relaxed),
            connects: SERIAL_CONNECTS.load(Ordering::Relaxed),
            disconnects: SERIAL_DISCONNECTS.load(Ordering::Relaxed),
            pico_events_emitted: SERIAL_PICO_EVENTS_EMITTED.load(Ordering::Relaxed),
            host_commands_written: SERIAL_HOST_COMMANDS_WRITTEN.load(Ordering::Relaxed),
        },
        audio: AudioDiagnostics {
            ticks: AUDIO_TICKS.load(Ordering::Relaxed),
            snapshots: AUDIO_SNAPSHOTS.load(Ordering::Relaxed),
            sessions_enumerated: AUDIO_SESSIONS_ENUMERATED.load(Ordering::Relaxed),
            slot_polls: AUDIO_SLOT_POLLS.load(Ordering::Relaxed),
            status_errors: AUDIO_STATUS_ERRORS.load(Ordering::Relaxed),
            empty_results: AUDIO_EMPTY_RESULTS.load(Ordering::Relaxed),
            updates_emitted: AUDIO_UPDATES_EMITTED.load(Ordering::Relaxed),
            pico_commands_sent: AUDIO_PICO_COMMANDS_SENT.load(Ordering::Relaxed),
        },
        com: ComDiagnostics {
            init_calls: COM_INIT_CALLS.load(Ordering::Relaxed),
            real_inits: COM_REAL_INITS.load(Ordering::Relaxed),
            reused_inits: COM_REUSED_INITS.load(Ordering::Relaxed),
            changed_mode_results: COM_CHANGED_MODE.load(Ordering::Relaxed),
            uninits: COM_UNINITS.load(Ordering::Relaxed),
        },
    }
}

pub fn record_com_init_call() {
    COM_INIT_CALLS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_com_real_init() {
    COM_REAL_INITS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_com_reused_init() {
    COM_REUSED_INITS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_com_changed_mode() {
    COM_CHANGED_MODE.fetch_add(1, Ordering::Relaxed);
}

pub fn record_com_uninit() {
    COM_UNINITS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_serial_connect() {
    SERIAL_CONNECTS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_serial_disconnect() {
    SERIAL_DISCONNECTS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_serial_bytes_read(bytes: usize, accumulator_len: usize, accumulator_cap: usize) {
    SERIAL_BYTES_READ.fetch_add(bytes as u64, Ordering::Relaxed);
    update_serial_accumulator(accumulator_len, accumulator_cap);
}

pub fn record_serial_message(accumulator_len: usize, accumulator_cap: usize) {
    SERIAL_MESSAGES.fetch_add(1, Ordering::Relaxed);
    update_serial_accumulator(accumulator_len, accumulator_cap);
}

pub fn record_serial_parse_error(accumulator_len: usize, accumulator_cap: usize) {
    SERIAL_PARSE_ERRORS.fetch_add(1, Ordering::Relaxed);
    update_serial_accumulator(accumulator_len, accumulator_cap);
}

pub fn record_serial_buffer_drop(accumulator_len: usize, accumulator_cap: usize) {
    SERIAL_BUFFER_DROPS.fetch_add(1, Ordering::Relaxed);
    update_serial_accumulator(accumulator_len, accumulator_cap);
}

pub fn record_serial_pico_event_emit() {
    SERIAL_PICO_EVENTS_EMITTED.fetch_add(1, Ordering::Relaxed);
}

pub fn record_serial_host_command_written() {
    SERIAL_HOST_COMMANDS_WRITTEN.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_tick() {
    AUDIO_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_snapshot(session_count: u64) {
    AUDIO_SNAPSHOTS.fetch_add(1, Ordering::Relaxed);
    AUDIO_SESSIONS_ENUMERATED.fetch_add(session_count, Ordering::Relaxed);
}

pub fn record_audio_slot_poll() {
    AUDIO_SLOT_POLLS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_status_error() {
    AUDIO_STATUS_ERRORS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_empty_result() {
    AUDIO_EMPTY_RESULTS.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_update_emit() {
    AUDIO_UPDATES_EMITTED.fetch_add(1, Ordering::Relaxed);
}

pub fn record_audio_pico_command_sent() {
    AUDIO_PICO_COMMANDS_SENT.fetch_add(1, Ordering::Relaxed);
}

fn update_serial_accumulator(len: usize, cap: usize) {
    SERIAL_ACCUMULATOR_LEN.store(len as u64, Ordering::Relaxed);
    SERIAL_ACCUMULATOR_CAP.store(cap as u64, Ordering::Relaxed);
    update_max(&SERIAL_ACCUMULATOR_MAX_LEN, len as u64);
}

fn update_max(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::Relaxed);
    while value > current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

fn process_snapshot() -> ProcessDiagnostics {
    let pid = std::process::id();
    let mut sys = sysinfo::System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]), true);

    if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid)) {
        let mem = proc.memory(); // In Bytes
        ProcessDiagnostics {
            pid,
            working_set_bytes: mem,
            peak_working_set_bytes: mem,
            private_usage_bytes: mem,
            peak_pagefile_usage_bytes: 0,
            pagefile_usage_bytes: 0,
            paged_pool_bytes: 0,
            nonpaged_pool_bytes: 0,
            page_faults: 0,
            handle_count: 0,
        }
    } else {
        ProcessDiagnostics {
            pid,
            working_set_bytes: 0,
            peak_working_set_bytes: 0,
            private_usage_bytes: 0,
            peak_pagefile_usage_bytes: 0,
            pagefile_usage_bytes: 0,
            paged_pool_bytes: 0,
            nonpaged_pool_bytes: 0,
            page_faults: 0,
            handle_count: 0,
        }
    }
}
