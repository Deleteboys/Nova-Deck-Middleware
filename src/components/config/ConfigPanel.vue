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

      <div class="pa-4 bg-black rounded-lg border border-zinc-700 mb-6">
        <div class="text-caption text-grey mb-2">Aktuelle Aktion:</div>
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

const store = useStreamDeckStore();
const buttonLabel = ref('');

const actionsLibrary = [
  { title: 'App Starten', icon: 'mdi-rocket-launch' },
  { title: 'Lautstärke +', icon: 'mdi-volume-plus' },
  { title: 'OBS Szene', icon: 'mdi-video-switch' },
  { title: 'Webseite', icon: 'mdi-earth' }
];

// Helper für die Anzeige
const currentIcon = computed(() => store.activeProfile?.keys[store.selectedElementId!]?.icon || 'mdi-plus');
const currentActionName = computed(() => store.activeProfile?.keys[store.selectedElementId!]?.action || 'Keine');

// Synchronisiere Input mit Store wenn Element gewechselt wird
// Synchronisiere Input mit Store, wenn das Element gewechselt wird
watch(() => store.selectedElementId, (newId) => {
  if (newId) {
    // WICHTIG: Hier muss .value stehen, NICHT .ref
    buttonLabel.value = store.activeProfile?.keys[newId]?.label || '';
  } else {
    buttonLabel.value = '';
  }
}, { immediate: true }); // immediate sorgt dafür, dass es auch beim ersten Klick sofort lädt

const saveChanges = () => {
  if (store.selectedElementId) {
    store.updateElementConfig(store.selectedElementId, { label: buttonLabel.value });
  }
};

const assignAction = (action: any) => {
  if (store.selectedElementId) {
    store.updateElementConfig(store.selectedElementId, {
      action: action.title,
      icon: action.icon
    });
  }
};
</script>

<style scoped>
.action-card { background: rgba(255,255,255,0.03) !important; border: 1px solid rgba(255,255,255,0.1); }
.action-card:hover:not(.v-list-item--disabled) { border-color: #6366f1; }
</style>