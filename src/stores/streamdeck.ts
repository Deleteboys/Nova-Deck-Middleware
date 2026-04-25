import { defineStore } from 'pinia'
import { watch } from 'vue'
import { setEffect, type LedEffectCommand, type TriggerType } from '@/services/streamdeckCommands'
import type { DeviceConfig } from '@/services/streamdeckCommands'

// Hilfsfunktion zur Konvertierung von Hex zu RGB
const hexToRgb = (hex: string) => {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return {r, g, b};
};

const rgbToHex = (r: number, g: number, b: number) =>
    `#${[r, g, b]
        .map((channel) => Math.max(0, Math.min(255, channel)).toString(16).padStart(2, '0'))
        .join('')}`;

// --- NEUE STRUKTUR FÜR MEHRFACH-BELEGUNGEN ---
export type ActionSetup = {
    action?: string;
    icon?: string;
    config?: any;
};

type ProfileKeyConfig = {
    label?: string;
    // Ein Objekt, das für jeden TriggerType (z.B. 'ShortPress', 'TurnRight') eine eigene Konfiguration hält
    actions?: Partial<Record<TriggerType, ActionSetup>>;
};

type Profile = {
    id: number;
    name: string;
    keys: Record<string, ProfileKeyConfig>;
};

export const useStreamDeckStore = defineStore('streamdeck', {
    state: () => ({
        currentProfileId: 0,
        selectedElementId: null as string | null,
        // Flag für ungespeicherte LED-Änderungen
        hasUnsavedLedChanges: false,
        profiles: [
            {id: 0, name: 'Main (Desktop)', keys: {}},
            {id: 1, name: 'Gaming', keys: {}},
            {id: 2, name: 'Streaming', keys: {}}
        ] as Profile[],
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
            reverse: false // <- Hier als Default hinzugefügt
        } as any
    }),

    getters: {
        activeProfile: (state) => state.profiles.find(p => p.id === state.currentProfileId)
    },

    actions: {
        setProfile(id: number) {
            this.currentProfileId = id;
            this.selectedElementId = null;
        },

        selectElement(id: string | null) {
            this.selectedElementId = this.selectedElementId === id ? null : id;
        },

        // --- NEUE MAPPING FUNKTIONEN ---

        // Aktualisiert nur das Text-Label des Buttons
        updateElementLabel(id: string | null, label: string) {
            if (!id || !this.activeProfile) return;

            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {};
            }
            this.activeProfile.keys[id].label = label;
        },

        // Speichert eine Aktion gezielt für einen bestimmten Auslöser (Trigger)
// Speichert eine Aktion gezielt für einen bestimmten Auslöser (Trigger)
        updateElementAction(id: string | null, trigger: TriggerType, setup: ActionSetup) {
            if (!id || !this.activeProfile) return;

            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {};
            }
            if (!this.activeProfile.keys[id].actions) {
                this.activeProfile.keys[id].actions = {};
            }

            this.activeProfile.keys[id].actions![trigger] = setup;
        },

        // Löscht nur die Aktion des ausgewählten Triggers, Label und andere Trigger bleiben erhalten
        clearElementAction(id: string | null, trigger: TriggerType) {
            if (!id || !this.activeProfile || !this.activeProfile.keys[id]?.actions) return;

            delete this.activeProfile.keys[id].actions![trigger];
        },


        // --- LED FUNKTIONEN ---

        applyDeviceConfig(config: DeviceConfig) {
            const ledEffect = config.led_effect;
            const [effectName, effectConfig] = Object.entries(ledEffect)[0] ?? [];

            if (!effectName || !effectConfig || typeof effectConfig !== 'object') {
                return;
            }

            const payload = effectConfig as Record<string, number | boolean>;

            const nextConfig: Record<string, unknown> = {
                ...this.ledConfig,
                effect: effectName,
            };

            if (
                'r' in payload &&
                'g' in payload &&
                'b' in payload &&
                typeof payload.r === 'number' &&
                typeof payload.g === 'number' &&
                typeof payload.b === 'number'
            ) {
                nextConfig.color = rgbToHex(payload.r, payload.g, payload.b);
            }

            for (const [key, value] of Object.entries(payload)) {
                if (key === 'r' || key === 'g' || key === 'b') continue;
                nextConfig[key] = value;
            }

            this.ledConfig = nextConfig;
            this.hasUnsavedLedChanges = false;
        },

        /**
         * Sendet die aktuellen ledConfig-Daten an das physikalische Streamdeck
         */
        async saveLedSettings() {
            const conf = this.ledConfig;
            const {r, g, b} = hexToRgb(conf.color);

            let command: LedEffectCommand;

            // Mapping auf das Rust/Pico Format (LedEffectCommand)
            // !!conf.reverse stellt sicher, dass es immer ein Boolean ist
            switch (conf.effect) {
                case 'Solid':
                    command = {Solid: {r, g, b, brightness: conf.brightness}};
                    break;
                case 'Blink':
                    command = {Blink: {r, g, b, brightness: conf.brightness, speed: conf.speed}};
                    break;
                case 'Rainbow':
                    command = {
                        Rainbow: {
                            brightness: conf.brightness,
                            speed: conf.speed,
                            saturation: conf.saturation,
                            reverse: !!conf.reverse
                        }
                    };
                    break;
                case 'Breathing':
                    command = {Breathing: {r, g, b, brightness: conf.brightness, speed: conf.speed}};
                    break;
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
                    };
                    break;
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
                    };
                    break;
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
                    };
                    break;
                case 'Aurora':
                    command = {Aurora: {brightness: conf.brightness, speed: conf.speed, reverse: !!conf.reverse}};
                    break;
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
                    };
                    break;
                case 'Astolfo':
                    command = {
                        Astolfo: {
                            brightness: conf.brightness,
                            speed: conf.speed,
                            saturation: conf.saturation,
                            spread: conf.spread,
                            reverse: !!conf.reverse
                        }
                    };
                    break;
                default:
                    console.error("Unbekannter Effekt:", conf.effect);
                    return;
            }

            try {
                await setEffect(command);
                this.hasUnsavedLedChanges = false;
                console.log("Hardware erfolgreich aktualisiert");
            } catch (error) {
                console.error("Fehler beim Senden an Pico:", error);
            }
        },

        // Hilfsmethode um den Watcher zu initialisieren (wird in App.vue aufgerufen)
        initHardwareWatcher() {
            watch(
                () => this.ledConfig,
                () => {
                    this.hasUnsavedLedChanges = true;
                },
                {deep: true}
            );
        }
    }
})
