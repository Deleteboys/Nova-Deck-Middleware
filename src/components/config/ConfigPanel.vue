<template>
  <div class="pa-0 d-flex flex-column fill-height bg-zinc-900 overflow-hidden">
    <div class="flex-grow-1 overflow-y-auto pa-4 custom-scrollbar">

      <section v-if="store.selectedElementId" class="mb-6">

        <!-- Direkt die Computed Properties und direkte Update-Funktionen nutzen -->
        <ConfigHeader
            :element-id="store.selectedElementId"
            :is-oled="isOledDisplay"
            :label="currentLabel"
            :icon="currentIcon"
            @update:label="updateLabel"
            @update:icon="updateIcon"
        />

        <v-divider class="mb-5 border-opacity-25" color="white"></v-divider>

        <OledSettings v-if="isOledDisplay" />
        <ActionBindings v-else />

      </section>

      <div v-else class="pa-10 text-center border-dashed rounded-lg border-zinc-700 text-grey mt-4">
        <v-icon icon="mdi-mouse-move-vertical" class="mb-2 opacity-50" size="large"></v-icon>
        <p class="text-body-2">Wähle ein Element auf dem Deck aus</p>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';

import ConfigHeader from './ConfigPanel/ConfigHeader.vue';
import OledSettings from './ConfigPanel/OledSettings.vue';
import ActionBindings from './ConfigPanel/ActionBindings.vue';

const store = useStreamDeckStore();

const isOledDisplay = computed(() => store.selectedElementId === 'oled-display');

const currentLabel = computed(() => {
  const id = store.selectedElementId;
  return id ? store.activeProfile?.keys[id]?.label || '' : '';
});

const currentIcon = computed(() => {
  const id = store.selectedElementId;
  return id ? store.activeProfile?.keys[id]?.icon || '' : '';
});

const updateLabel = (val: string) => {
  if (store.selectedElementId && !isOledDisplay.value) {
    store.updateElementLabel(store.selectedElementId, val);
  }
};

const updateIcon = (val: string) => {
  if (store.selectedElementId && store.activeProfile?.keys[store.selectedElementId]) {
    store.updateElementIcon(store.selectedElementId, val);
  }
};
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(255, 255, 255, 0.2); }
</style>