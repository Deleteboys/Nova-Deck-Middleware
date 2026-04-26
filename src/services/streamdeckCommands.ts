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

export type ActionConfig =
    | { type: 'PressKey'; key: string }
    | { type: 'SpotifyVolume'; step: number }
    | { type: 'ToggleAudio'; device1: string; device2: string }
    | { type: 'MasterVolume'; step: number }
    | { type: 'ToggleAppAudio'; process_name: string };

export type ProcessInfo = {
    pid: number;
    name: string;
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
export async function syncActionMappings(mappings: MappingPayload[]): Promise<void> {
    try {
        await invoke("sync_mappings", {mappings});
    } catch (error) {
        console.error("Fehler beim Synchronisieren der Mappings in Rust:", error);
        throw error;
    }
}
