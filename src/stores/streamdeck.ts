import { defineStore } from 'pinia'

type ProfileKeyConfig = {
    value?: number;
};

type Profile = {
    id: number;
    name: string;
    keys: Record<string, ProfileKeyConfig>;
};

type StreamdeckState = {
    currentProfileId: number;
    selectedElementId: string | null;
    profiles: Profile[];
};

export const useStreamDeckStore = defineStore('streamdeck', {
    state: (): StreamdeckState => ({
        // Globaler State der gesamten App
        currentProfileId: 0,
        selectedElementId: null as string | null,

        profiles: [
            { id: 0, name: 'Main (Desktop)', keys: {} },
            { id: 1, name: 'Gaming', keys: {} },
            { id: 2, name: 'Streaming', keys: {} }
        ]
    }),

    getters: {
        activeProfile: (state) => state.profiles.find(p => p.id === state.currentProfileId)
    },

    actions: {
        setProfile(id: number) {
            this.currentProfileId = id;
            this.selectedElementId = null; // Auswahl zurücksetzen bei Profilwechsel
        },
        selectElement(id: string) {
            this.selectedElementId = id;
        }
    }
})
