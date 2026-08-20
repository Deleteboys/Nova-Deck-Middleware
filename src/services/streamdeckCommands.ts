import {invoke} from "@tauri-apps/api/core";

// --- TYPEN FÜR RUST-KOMMUNIKATION ---

export type TriggerType =
    | 'ShortPress'
    | 'LongPress'
    | 'DoublePress'
    | 'TurnLeft'
    | 'TurnRight'
    | 'PushTurnLeft'
    | 'PushTurnRight'
    | 'PushPress';

export type AudioDeviceInfo = {
    id: string;
    name: string;
};

export type AppSwitcherEntry = {
    process_name: string;
    icon?: string | null;
};

export type ActionConfig =
    | { type: 'PressKey'; key: string }
    | { type: 'CustomMacro'; key: string }
    | { type: 'MediaControl'; key: string }
    | { type: 'SpotifyVolume'; step: number }
    | { type: 'MasterVolume'; step: number; snap?: boolean }
    | { type: 'ToggleAppAudio'; process_name: string; use_switcher?: boolean }
    | { type: 'ToggleMasterMute' }
    | { type: 'AppVolume'; process_name: string; step: number; use_switcher?: boolean; snap?: boolean }
    | { type: 'ForegroundVolume'; step: number; snap?: boolean }
    | { type: 'ToggleForegroundAudio' }
    | { type: 'SwitchAudioDevice'; device1: string; device2: string }
    | { type: 'SpotifyLikeAction' }
    | { type: 'AppSwitcherCycle'; apps: AppSwitcherEntry[]; shared_icon?: string | null; direction: number; hide_closed_apps?: boolean };

export type FirmwareUpdateInfo = {
    version: string;
    download_url: string;
};

export type ProcessInfo = {
    pid: number;
    name: string;
};

export type RuntimeDiagnostics = {
    uptime_secs: number;
    process: {
        pid: number;
        working_set_bytes: number;
        peak_working_set_bytes: number;
        private_usage_bytes: number;
        peak_pagefile_usage_bytes: number;
        pagefile_usage_bytes: number;
        paged_pool_bytes: number;
        nonpaged_pool_bytes: number;
        page_faults: number;
        handle_count: number;
    };
    serial: {
        bytes_read: number;
        messages_decoded: number;
        parse_errors: number;
        buffer_drops: number;
        accumulator_len: number;
        accumulator_capacity: number;
        accumulator_max_len: number;
        connects: number;
        disconnects: number;
        pico_events_emitted: number;
        host_commands_written: number;
    };
    audio: {
        ticks: number;
        snapshots: number;
        sessions_enumerated: number;
        slot_polls: number;
        status_errors: number;
        empty_results: number;
        updates_emitted: number;
        pico_commands_sent: number;
    };
    com: {
        init_calls: number;
        real_inits: number;
        reused_inits: number;
        changed_mode_results: number;
        uninits: number;
    };
};

export type LedEffectCommand =
    | {
    Solid: {
        r: number;
        g: number;
        b: number;
        brightness: number;
    };
}
    | {
    Blink: {
        r: number;
        g: number;
        b: number;
        brightness: number;
        speed: number;
    };
}
    | {
    Rainbow: {
        brightness: number;
        speed: number;
        saturation: number;
        reverse: boolean;
    };
}
    | {
    Breathing: {
        r: number;
        g: number;
        b: number;
        brightness: number;
        speed: number;
    };
}
    | {
    Chase: {
        r: number;
        g: number;
        b: number;
        brightness: number;
        speed: number;
        size: number;
        reverse: boolean;
    };
}
    | {
    Comet: {
        r: number;
        g: number;
        b: number;
        brightness: number;
        speed: number;
        tail: number;
        reverse: boolean;
    };
}
    | {
    Sparkle: {
        r: number;
        g: number;
        b: number;
        brightness: number;
        speed: number;
        density: number;
    };
}
    | {
    Aurora: {
        brightness: number;
        speed: number;
        reverse: boolean;
    };
}
    | {
    ColorOrbit: {
        hue: number;
        hue_shift: number;
        saturation: number;
        brightness: number;
        speed: number;
        reverse: boolean;
    };
}
    | {
    Astolfo: {
        brightness: number;
        speed: number;
        saturation: number;
        spread: number;
        reverse: boolean;
    };
};

export type DeviceConfig = {
    led_effect: LedEffectCommand;
};

type HostToPicoCommand =
    | "Ping"
    | "StartBootloader"
    | "GetConfig"
    | "GetVersion"
    | {
    FillAll: {
        r: number;
        g: number;
        b: number;
        brightness: number;
    };
}
    | {
    SetEffect: {
        effect: LedEffectCommand;
    };
}
    | {
    SetLed: {
        index: number;
        r: number;
        g: number;
        b: number;
        brightness: number;
    };
};

// --- LED UND HARDWARE API ---

async function sendToPico(command: HostToPicoCommand): Promise<void> {
    await invoke("send_to_pico", {command});
}

export async function setLed(
    index: number,
    rgb: { r: number; g: number; b: number },
    brightness = 200
): Promise<void> {
    await sendToPico({
        SetLed: {
            index,
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            brightness,
        },
    });
}

export async function fillAll(
    rgb: { r: number; g: number; b: number },
    brightness = 200
): Promise<void> {
    await sendToPico({
        FillAll: {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            brightness,
        },
    });
}

export async function setEffect(effect: LedEffectCommand): Promise<void> {
    await sendToPico({
        SetEffect: {effect},
    });
}

export async function startBootloader(): Promise<void> {
    await sendToPico("StartBootloader");
}

export async function requestDeviceConfig(): Promise<void> {
    await sendToPico("GetConfig");
}

export async function getConnectionStatus(): Promise<boolean> {
    return invoke<boolean>("get_connection_status");
}

// --- ACTION UND MAPPING API ---

export async function updateActionMapping(
    elementId: string,
    triggerType: TriggerType,
    actionConfig: ActionConfig
): Promise<void> {
    try {
        await invoke("update_mapping", {
            payload: {
                element_id: elementId,
                trigger_type: triggerType,
                action_config: actionConfig
            }
        });
    } catch (error) {
        console.error("Fehler beim Senden des Mappings an Rust:", error);
        throw error;
    }
}

export async function removeActionMapping(
    elementId: string,
    triggerType: TriggerType
): Promise<void> {
    try {
        await invoke("remove_mapping", {
            payload: {
                element_id: elementId,
                trigger_type: triggerType
            }
        });
    } catch (error) {
        console.error("Fehler beim Löschen des Mappings in Rust:", error);
        throw error;
    }
}

type MappingPayload = {
    element_id: string;
    trigger_type: TriggerType;
    action_config: ActionConfig;
};

export async function getActiveProcesses(): Promise<string[]> {
    try {
        return await invoke<string[]>("get_active_processes");
    } catch (error) {
        console.error("Fehler beim Abrufen der aktiven Prozesse in Rust:", error);
        throw error;
    }
}

export async function getActiveAudioProcesses(): Promise<string[]> {
    try {
        return await invoke<string[]>("get_active_audio_processes");
    } catch (error) {
        console.error("Fehler beim Abrufen der aktiven Prozesse in Rust:", error);
        throw error;
    }
}

export async function syncActionMappings(mappings: MappingPayload[]): Promise<void> {
    try {
        await invoke("sync_mappings", {mappings});
    } catch (error) {
        console.error("Fehler beim Synchronisieren der Mappings in Rust:", error);
        throw error;
    }
}

export async function setIconSlot(slot: number, icon: string): Promise<void> {
    try {
        await invoke("set_icon_slot", { slot, icon });
    } catch (error) {
        console.error(`Fehler beim Senden von Icon ${icon} für Slot ${slot} an Rust:`, error);
        throw error;
    }
}

export async function checkFirmwareUpdate(): Promise<FirmwareUpdateInfo | null> {
    try {
        return await invoke<FirmwareUpdateInfo | null>("check_firmware_update");
    } catch (error) {
        console.error("Fehler beim Prüfen auf Firmware-Updates:", error);
        return null;
    }
}

export async function downloadAndFlashFirmware(downloadUrl: string): Promise<void> {
    await invoke("download_and_flash_firmware", { downloadUrl });
}

export async function requestFirmwareVersion(): Promise<void> {
    await sendToPico("GetVersion");
}

export async function setStartMinimized(value: boolean): Promise<void> {
    await invoke("set_start_minimized", { value });
}

export async function getStartMinimized(): Promise<boolean> {
    return await invoke("get_start_minimized");
}

export async function getAudioDevices(): Promise<AudioDeviceInfo[]> {
    try {
        return await invoke<AudioDeviceInfo[]>("get_audio_output_devices");
    } catch (error) {
        console.error("Fehler beim Abrufen der Audiogeräte:", error);
        return [];
    }
}

export async function startSpotifyLogin(clientId: string): Promise<void> {
    try {
        await invoke("start_spotify_login", { clientId });
    } catch (error) {
        console.error("Fehler beim Spotify Login:", error);
        throw error;
    }
}

export async function getRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
    return await invoke<RuntimeDiagnostics>("get_runtime_diagnostics");
}
