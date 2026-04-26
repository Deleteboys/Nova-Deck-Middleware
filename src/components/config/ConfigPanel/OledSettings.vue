<template>
  <div class="mb-6">
    <div class="text-body-2 text-grey font-weight-medium mb-4">Display Slots konfigurieren</div>

    <div class="d-flex flex-column gap-3">
      <v-card
          v-for="(slot, index) in oledSlots"
          :key="'slot-' + index"
          color="#18181b"
          variant="flat"
          class="border border-zinc-800 rounded-lg overflow-hidden"
      >
        <div class="d-flex align-center px-3 py-2 bg-zinc-800 border-b border-zinc-700">
          <v-icon icon="mdi-monitor-dashboard" color="primary" size="small" class="mr-2"></v-icon>
          <span class="text-body-2 font-weight-bold text-white">Slot {{ index + 1 }}</span>
        </div>

        <div class="px-4 py-3">

          <div class="d-flex align-center justify-space-between mb-4">
            <div class="text-body-2 text-grey">Anzeigesymbol</div>
            <div style="width: 130px;">
              <v-select
                  v-model="slot.icon"
                  :items="['MASTER', 'SPOTIFY', 'DISCORD', 'BROWSER', 'NONE']"
                  variant="underlined"
                  density="compact"
                  hide-details
                  class="slot-select text-white"
                  @update:model-value="(newIcon) => handleIconChange(index, newIcon)"
              ></v-select>
            </div>
          </div>

          <div>
            <div class="d-flex justify-space-between align-center mb-1">
              <div class="text-body-2 text-grey">Gekoppelter Prozess</div>
              <v-btn
                  icon="mdi-refresh"
                  variant="text"
                  size="x-small"
                  color="grey"
                  title="Liste aktualisieren"
                  @click="fetchProcesses"
              ></v-btn>
            </div>
            <v-autocomplete
                v-model="slot.process"
                :items="activeProcesses"
                variant="underlined"
                density="compact"
                hide-details
                placeholder="Kein Prozess (Standard)"
                class="text-white"
                @update:model-value="updateOledConfig"
            ></v-autocomplete>
          </div>

        </div>
      </v-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import { getActiveProcesses, setIconSlot } from '@/services/streamdeckCommands';

const store = useStreamDeckStore();

const oledSlots = ref([
  { icon: 'MASTER', process: '' },
  { icon: 'SPOTIFY', process: '' },
  { icon: 'DISCORD', process: '' },
  { icon: 'BROWSER', process: '' }
]);

const activeProcesses = ref<string[]>([]);

const fetchProcesses = async () => {
  try {
    activeProcesses.value = await getActiveProcesses();
  } catch (error) {
    console.error("Prozesse konnten nicht geladen werden:", error);
  }
};

const updateOledConfig = () => {
  if (!store.activeProfile) return;
  if (!store.activeProfile.keys['oled-display']) store.activeProfile.keys['oled-display'] = {};
  store.activeProfile.keys['oled-display'].slots = JSON.parse(JSON.stringify(oledSlots.value));
};

/**
 * Verarbeitet die Änderung eines Icons:
 * 1. Aktualisiert die lokale Konfiguration im Store/LocalStorage.
 * 2. Sendet den neuen Icon-Typ für den spezifischen Slot an den Pico.
 */
const handleIconChange = async (index: number, newIcon: string) => {
  updateOledConfig();

  try {
    // index entspricht dem Slot (0-3) auf dem Pico
    await setIconSlot(index, newIcon);
  } catch (error) {
    console.error("Fehler beim Hardware-Update des Icons:", error);
  }
};

// Synchronisation mit dem Store bei Element-Auswahl
watch(() => store.selectedElementId, (newId) => {
  if (newId === 'oled-display') {
    const savedSlots = store.activeProfile?.keys['oled-display']?.slots;
    if (savedSlots && savedSlots.length === 4) {
      oledSlots.value = JSON.parse(JSON.stringify(savedSlots));
    } else {
      oledSlots.value = [
        { icon: 'MASTER', process: '' },
        { icon: 'SPOTIFY', process: '' },
        { icon: 'DISCORD', process: '' },
        { icon: 'BROWSER', process: '' }
      ];
    }
  }
}, { immediate: true });

onMounted(() => {
  fetchProcesses();
});
</script>

<style scoped>
.gap-3 { gap: 12px; }

.slot-select :deep(.v-field__input) {
  font-size: 0.875rem !important;
  text-align: right;
  color: #ffffff !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}
.slot-select :deep(.v-field__append-inner) {
  padding-top: 0 !important;
  align-items: center;
}
</style>