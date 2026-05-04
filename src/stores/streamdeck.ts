import {defineStore} from 'pinia'
import {watch} from 'vue'
import {
    setEffect,
    syncActionMappings,
    type ActionConfig,
    type LedEffectCommand,
    type TriggerType, setIconSlot
} from '@/services/streamdeckCommands'
import type {DeviceConfig} from '@/services/streamdeckCommands'
import {invoke} from "@tauri-apps/api/core";

const hexToRgb = (hex: string) => {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    return {r, g, b}
}

const rgbToHex = (r: number, g: number, b: number) =>
    `#${[r, g, b]
        .map((channel) => Math.max(0, Math.min(255, channel)).toString(16).padStart(2, '0'))
        .join('')}`

// --- NEUE TYPEN FÜR OLED SLOTS ---
export interface OledSlot {
    icon: string;
    process: string;
    muted?: boolean;
    value?: number;
}

export type ActionSetup = {
    action?: string
    icon?: string
    config?: any
}

type ProfileKeyConfig = {
    label?: string
    icon?: string
    value?: number
    actions?: Partial<Record<TriggerType, ActionSetup>>
    // HIER: Slots als optionales Feld hinzufügen, damit TS nicht mehr meckert
    slots?: OledSlot[]
}

type Profile = {
    id: number
    name: string
    keys: Record<string, ProfileKeyConfig>
}

type PersistedState = {
    currentProfileId: number
    profiles: Profile[]
}

const STORAGE_KEY = 'streamdeck-state-v1'

const DEFAULT_PROFILES: Profile[] = [
    {id: 0, name: 'Main', keys: {}},
    {id: 1, name: 'Profile 2', keys: {}},
]

const isValidPersistedState = (value: unknown): value is PersistedState => {
    if (!value || typeof value !== 'object') return false
    const candidate = value as PersistedState
    return typeof candidate.currentProfileId === 'number' && Array.isArray(candidate.profiles)
}

const loadPersistedState = (): PersistedState | null => {
    if (typeof window === 'undefined') return null

    try {
        const raw = window.localStorage.getItem(STORAGE_KEY)
        if (!raw) return null

        const parsed: unknown = JSON.parse(raw)
        return isValidPersistedState(parsed) ? parsed : null
    } catch (error) {
        console.warn('Persisted StreamDeck config could not be loaded:', error)
        return null
    }
}

const isActionConfig = (value: unknown): value is ActionConfig => {
    if (!value || typeof value !== 'object') return false
    const config = value as { type?: string }

    return (
        config.type === 'PressKey' ||
        config.type === 'MediaControl' ||
        config.type === 'SpotifyVolume' ||
        config.type === 'ToggleAudio' ||
        config.type === 'MasterVolume' ||
        config.type === 'ToggleAppAudio' ||
        config.type === 'MasterVolume' ||
        config.type === 'ToggleMasterMute' ||
        config.type === 'AppVolume' ||
        config.type === 'ForegroundVolume' ||
        config.type === 'ToggleForegroundAudio'
    )
}

const persistedState = loadPersistedState()

export const useStreamDeckStore = defineStore('streamdeck', {
    state: () => ({
        currentProfileId: persistedState?.currentProfileId ?? 0,
        selectedElementId: null as string | null,
        isDeviceConnected: false,
        hasUnsavedLedChanges: false,
        suppressLedDirtyTracking: false,
        profiles: persistedState?.profiles ?? DEFAULT_PROFILES,
        ledConfig: {
            effect: 'ColorOrbit',
            color: '#00e5ff',
            brightness: 255,
            speed: 100,
            size: 4,
            tail: 80,
            density: 100,
            hue: 200,
            hue_shift: 50,
            saturation: 255,
            spread: 120,
            reverse: false
        } as any
    }),

    getters: {
        activeProfile: (state) => state.profiles.find((p) => p.id === state.currentProfileId)
    },

    actions: {
        persistState() {
            if (typeof window === 'undefined') return

            const stateToPersist: PersistedState = {
                currentProfileId: this.currentProfileId,
                profiles: this.profiles
            }

            try {
                window.localStorage.setItem(STORAGE_KEY, JSON.stringify(stateToPersist))
            } catch (error) {
                console.warn('Persisted StreamDeck config could not be saved:', error)
            }
        },

        async syncOledIconsToBackend() {
            const slots = this.activeProfile?.keys['oled-display']?.slots;
            if (!slots) return;

            for (let i = 0; i < slots.length; i++) {
                try {
                    await setIconSlot(i, slots[i].icon);
                    // Kurze Pause, um den seriellen Puffer nicht zu überlasten
                    await new Promise(resolve => setTimeout(resolve, 50));
                } catch (e) {
                    console.error(`Fehler beim Sync von Slot ${i}:`, e);
                }
            }
        },
        async syncMonitorMappingsToBackend() {
            const slots = this.activeProfile?.keys['oled-display']?.slots;
            if (!slots) return;

            for (let i = 0; i < slots.length; i++) {
                try {
                    await invoke("update_monitor_mapping", {
                        slot: i,
                        processName: slots[i].process || ""
                    });
                } catch (e) {
                    console.error(`Fehler beim Sync des Monitors für Slot ${i}:`, e);
                }
            }
        },
        async syncActiveProfileMappingsToBackend() {
            if (!this.activeProfile) return

            const mappings: { element_id: string; trigger_type: TriggerType; action_config: ActionConfig }[] = []

            for (const [elementId, keyConfig] of Object.entries(this.activeProfile.keys)) {
                if (!keyConfig.actions) continue

                for (const [triggerType, setup] of Object.entries(keyConfig.actions)) {
                    if (!setup?.config || !isActionConfig(setup.config)) continue

                    mappings.push({
                        element_id: elementId,
                        trigger_type: triggerType as TriggerType,
                        action_config: setup.config
                    })
                }
            }

            await syncActionMappings(mappings)
        },

        async setDeviceConnected(isConnected: boolean) {
            this.isDeviceConnected = isConnected
            if (isConnected) {
                console.log("Gerät verbunden - starte Voll-Sync...");
                await this.syncOledIconsToBackend()
                await this.syncMonitorMappingsToBackend();
            }
        },

        setProfile(id: number) {
            this.currentProfileId = id
            this.selectedElementId = null
            this.persistState()
        },

        selectElement(id: string | null) {
            this.selectedElementId = this.selectedElementId === id ? null : id
        },

        updateElementLabel(id: string | null, label: string) {
            if (!id || !this.activeProfile) return

            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {}
            }
            this.activeProfile.keys[id].label = label
            this.persistState()
        },

        updateElementIcon(id: string | null, icon: string) {
            if (!id || !this.activeProfile) return

            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {}
            }

            this.activeProfile.keys[id].icon = icon
            this.persistState()
        },

        updateElementAction(id: string | null, trigger: TriggerType, setup: ActionSetup) {
            if (!id || !this.activeProfile) return

            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {}
            }
            if (!this.activeProfile.keys[id].actions) {
                this.activeProfile.keys[id].actions = {}
            }

            this.activeProfile.keys[id].actions![trigger] = setup
            this.persistState()
        },

        // HIER: updateOledSlots mit korrektem Typ-Handling
        updateOledSlots(slots: OledSlot[]) {
            if (!this.activeProfile) return;

            if (!this.activeProfile.keys['oled-display']) {
                this.activeProfile.keys['oled-display'] = {};
            }

            // Hier wird .slots nun von TypeScript erkannt
            this.activeProfile.keys['oled-display'].slots = JSON.parse(JSON.stringify(slots));
            this.persistState();
        },

        clearElementAction(id: string | null, trigger: TriggerType) {
            if (!id || !this.activeProfile || !this.activeProfile.keys[id]?.actions) return

            delete this.activeProfile.keys[id].actions![trigger]
            this.persistState()
        },

        applyDeviceConfig(config: DeviceConfig) {
            const ledEffect = config.led_effect
            const [effectName, effectConfig] = Object.entries(ledEffect)[0] ?? []

            if (!effectName || !effectConfig || typeof effectConfig !== 'object') {
                return
            }

            this.suppressLedDirtyTracking = true

            const payload = effectConfig as Record<string, number | boolean>

            const nextConfig: Record<string, unknown> = {
                ...this.ledConfig,
                effect: effectName
            }

            if (
                'r' in payload &&
                'g' in payload &&
                'b' in payload &&
                typeof payload.r === 'number' &&
                typeof payload.g === 'number' &&
                typeof payload.b === 'number'
            ) {
                nextConfig.color = rgbToHex(payload.r, payload.g, payload.b)
            }

            for (const [key, value] of Object.entries(payload)) {
                if (key === 'r' || key === 'g' || key === 'b') continue
                nextConfig[key] = value
            }

            this.ledConfig = nextConfig
            this.hasUnsavedLedChanges = false
            Promise.resolve().then(() => {
                this.suppressLedDirtyTracking = false
            })
        },

        async saveLedSettings() {
            const conf = this.ledConfig
            const {r, g, b} = hexToRgb(conf.color)

            let command: LedEffectCommand

            switch (conf.effect) {
                case 'Solid':
                    command = {Solid: {r, g, b, brightness: conf.brightness}}
                    break
                case 'Blink':
                    command = {Blink: {r, g, b, brightness: conf.brightness, speed: conf.speed}}
                    break
                case 'Rainbow':
                    command = {
                        Rainbow: {
                            brightness: conf.brightness,
                            speed: conf.speed,
                            saturation: conf.saturation,
                            reverse: !!conf.reverse
                        }
                    }
                    break
                case 'Breathing':
                    command = {Breathing: {r, g, b, brightness: conf.brightness, speed: conf.speed}}
                    break
                case 'Chase':
                    command = {
                        Chase: {
                            r,
                            g,
                            b,
                            brightness: conf.brightness,
                            speed: conf.speed,
                            size: conf.size,
                            reverse: !!conf.reverse
                        }
                    }
                    break
                case 'Comet':
                    command = {
                        Comet: {
                            r,
                            g,
                            b,
                            brightness: conf.brightness,
                            speed: conf.speed,
                            tail: conf.tail,
                            reverse: !!conf.reverse
                        }
                    }
                    break
                case 'Sparkle':
                    command = {
                        Sparkle: {
                            r,
                            g,
                            b,
                            brightness: conf.brightness,
                            speed: conf.speed,
                            density: conf.density
                        }
                    }
                    break
                case 'Aurora':
                    command = {Aurora: {brightness: conf.brightness, speed: conf.speed, reverse: !!conf.reverse}}
                    break
                case 'ColorOrbit':
                    command = {
                        ColorOrbit: {
                            hue: conf.hue,
                            hue_shift: conf.hue_shift,
                            saturation: conf.saturation,
                            brightness: conf.brightness,
                            speed: conf.speed,
                            reverse: !!conf.reverse
                        }
                    }
                    break
                case 'Astolfo':
                    command = {
                        Astolfo: {
                            brightness: conf.brightness,
                            speed: conf.speed,
                            saturation: conf.saturation,
                            spread: conf.spread,
                            reverse: !!conf.reverse
                        }
                    }
                    break
                default:
                    console.error('Unknown effect:', conf.effect)
                    return
            }

            try {
                await setEffect(command)
                this.hasUnsavedLedChanges = false
                console.log('Hardware updated')
            } catch (error) {
                console.error('Error sending to pico:', error)
            }
        },

        initHardwareWatcher() {
            watch(
                () => this.ledConfig,
                () => {
                    if (this.suppressLedDirtyTracking) {
                        return
                    }
                    this.hasUnsavedLedChanges = true
                },
                {deep: true}
            )
        }
    }
})