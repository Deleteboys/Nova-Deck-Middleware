<template>
  <div class="pa-4 d-flex flex-column fill-height bg-zinc-900">

    <div v-if="store.selectedElementId" class="mb-6">
      <h3 class="text-subtitle-2 mb-4 text-primary">KONFIGURATION: {{ store.selectedElementId }}</h3>

      <v-text-field
          v-model="buttonLabel"
          label="Button Beschriftung"
          variant="outlined"
          density="comfortable"
          class="mb-4"
          @input="saveChanges"
      ></v-text-field>

      <v-select
          v-model="selectedTriggerType"
          :items="triggerOptions"
          label="Auslöser (Event)"
          variant="outlined"
          density="comfortable"
          class="mb-4"
      ></v-select>

      <div class="pa-4 bg-black rounded-lg border border-zinc-700 mb-6">
        <div class="d-flex justify-space-between align-center mb-2">
          <div class="text-caption text-grey">Aktuelle Aktion:</div>
          <v-btn
              v-if="currentActionName !== 'Keine'"
              size="x-small"
              color="error"
              variant="text"
              icon="mdi-delete"
              @click="unbindAction"
          ></v-btn>
        </div>
        <div class="d-flex align-center">
          <v-icon :icon="currentIcon" color="primary" class="mr-3"></v-icon>
          <span class="text-body-2">{{ currentActionName }}</span>
        </div>
      </div>
    </div>

    <div v-else class="pa-10 text-center border-dashed rounded-lg text-grey mb-6">
      Wähle ein Element auf dem Deck aus
    </div>

    <v-divider class="mb-6"></v-divider>

    <div class="flex-grow-1 overflow-y-auto">
      <div class="text-overline mb-4 text-grey">Aktionen Bibliothek</div>
      <v-list bg-color="transparent" density="compact" nav>
        <v-list-item
            v-for="a in actionsLibrary"
            :key="a.title"
            :prepend-icon="a.icon"
            :title="a.title"
            :disabled="!store.selectedElementId"
            @click="assignAction(a)"
            class="mb-2 rounded-lg action-card"
        >
        </v-list-item>
      </v-list>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import { updateActionMapping, removeActionMapping } from '@/services/streamdeckCommands';

const store = useStreamDeckStore();
const buttonLabel = ref('');
const selectedTriggerType = ref('ShortPress');

// Die Bibliothek nutzt jetzt das saubere Config-Format für Rust
const actionsLibrary = [
  {
    title: 'Taste F14 drücken',
    icon: 'mdi-keyboard',
    config: { type: 'PressKey', key: 'F14' }
  },
  {
    title: 'Taste F15 drücken',
    icon: 'mdi-keyboard',
    config: { type: 'PressKey', key: 'F15' }
  },
  {
    title: 'Spotify Volume 50%',
    icon: 'mdi-volume-plus',
    config: { type: 'SpotifyVolume', volume: 50 }
  },
  {
    title: 'Audio Toggle',
    icon: 'mdi-swap-horizontal',
    config: { type: 'ToggleAudio', device1: 'HyperX', device2: 'Speakers' }
  }
];

// Dynamische Optionen basierend auf Button oder Encoder
const triggerOptions = computed(() => {
  if (store.selectedElementId?.startsWith('enc-')) {
    return [
      { title: 'Nach Rechts drehen', value: 'TurnRight' },
      { title: 'Nach Links drehen', value: 'TurnLeft' },
      { title: 'Drücken', value: 'PushPress' }
    ];
  }
  return [
    { title: 'Kurz drücken', value: 'ShortPress' },
    { title: 'Lang drücken', value: 'LongPress' },
    { title: 'Doppelklick', value: 'DoublePress' }
  ];
});

const currentIcon = computed(() => store.activeProfile?.keys[store.selectedElementId!]?.icon || 'mdi-plus');
const currentActionName = computed(() => store.activeProfile?.keys[store.selectedElementId!]?.action || 'Keine');

// Synchronisiere UI mit dem Store bei Auswahl eines Elements
watch(() => store.selectedElementId, (newId) => {
  if (newId) {
    buttonLabel.value = store.activeProfile?.keys[newId]?.label || '';
    selectedTriggerType.value = newId.startsWith('enc-') ? 'TurnRight' : 'ShortPress';
  } else {
    buttonLabel.value = '';
  }
}, { immediate: true });

const saveChanges = () => {
  if (store.selectedElementId) {
    store.updateElementConfig(store.selectedElementId, { label: buttonLabel.value });
  }
};

const assignAction = async (action: any) => {
  if (store.selectedElementId) {
    // Im Frontend speichern
    store.updateElementConfig(store.selectedElementId, {
      action: action.title,
      icon: action.icon,
      config: action.config
    });

    // Ans Backend schicken
    try {
      await updateActionMapping(
          store.selectedElementId,
          selectedTriggerType.value,
          action.config
      );
    } catch (e) {
      console.error("Mapping Fehler:", e);
    }
  }
};

const unbindAction = async () => {
  if (store.selectedElementId) {
    // Im Frontend löschen
    store.clearElementAction(store.selectedElementId);

    // Im Backend löschen
    try {
      await removeActionMapping(
          store.selectedElementId,
          selectedTriggerType.value
      );
    } catch (e) {
      console.error("Unbind Fehler:", e);
    }
  }
};
</script>

<style scoped>
.action-card {
  background: rgba(255,255,255,0.03) !important;
  border: 1px solid rgba(255,255,255,0.1);
  transition: border-color 0.2s;
}
.action-card:hover:not(.v-list-item--disabled) {
  border-color: #3b82f6;
}
</style>