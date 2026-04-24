import { defineStore } from 'pinia'

type ProfileKeyConfig = {
    value?: number;
    label?: string;
    icon?: string;
    action?: string;
    actionValue?: string;
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
        profiles: [
            { id: 0, name: 'Main (Desktop)', keys: {} },
            { id: 1, name: 'Gaming', keys: {} },
            { id: 2, name: 'Streaming', keys: {} }
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
            spread: 120
        }
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

        // Diese Action MUSS exakt so hier stehen
        updateElementConfig(id: string | null, updates: Partial<ProfileKeyConfig>) {
            if (!id || !this.activeProfile) return;

            // Falls der Key noch nicht existiert, Objekt erstellen
            if (!this.activeProfile.keys[id]) {
                this.activeProfile.keys[id] = {};
            }

            // Bestehende Daten mit neuen Updates mergen
            this.activeProfile.keys[id] = {
                ...this.activeProfile.keys[id],
                ...updates
            };

            console.log("Gespeichert:", id, this.activeProfile.keys[id]);
        }
    }
})