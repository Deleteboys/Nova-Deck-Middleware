<template>
  <div class="pa-0 d-flex flex-column fill-height bg-zinc-900 overflow-hidden">
    <div class="flex-grow-1 overflow-y-auto pa-4 custom-scrollbar">

      <section v-if="store.selectedElementId" class="mb-6">

        <ConfigHeader
            :element-id="store.selectedElementId"
            :is-oled="isOledDisplay"
            v-model:label="buttonLabel"
            @save="saveChanges"
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
import { computed, ref, watch } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';

// Unterkomponenten importieren
import ConfigHeader from './ConfigPanel/ConfigHeader.vue';
import OledSettings from './ConfigPanel/OledSettings.vue';
import ActionBindings from './ConfigPanel/ActionBindings.vue';

const store = useStreamDeckStore();
const buttonLabel = ref('');

const isOledDisplay = computed(() => store.selectedElementId === 'oled-display');

// Synchronisation des Labels bei Auswahl eines neuen Elements
watch(() => store.selectedElementId, (newId) => {
  if (newId) {
    buttonLabel.value = store.activeProfile?.keys[newId]?.label || '';
  }
}, { immediate: true });

const saveChanges = () => {
  if (store.selectedElementId && !isOledDisplay.value) {
    store.updateElementLabel(store.selectedElementId, buttonLabel.value);
  }
};
</script>

<style scoped>
/* Layout & Scrollbar Styles */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>