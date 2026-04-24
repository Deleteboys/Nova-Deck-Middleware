<template>
  <v-card class="hardware-board pa-10" elevation="24" rounded="xl" color="#18181b" border>
    <div class="d-flex flex-column align-center">

      <div class="mb-8">
        <OledDisplay />
      </div>

      <div class="d-flex justify-center gap-8 mb-10">
        <div
            v-for="n in 4"
            :key="`enc-${n}`"
            class="encoder"
            @click="store.selectElement('enc-' + (n-1))"
        >
          <div :class="['encoder-inner', { active: store.selectedElementId === 'enc-' + (n-1) }]">
            <div class="encoder-notch"></div>
          </div>
        </div>
      </div>

      <div class="button-grid">
        <v-card
            v-for="n in 8"
            :key="`btn-${n}`"
            :class="['stream-btn', { selected: store.selectedElementId === 'btn-' + (n-1) }]"
            color="#27272a"
            elevation="4"
            rounded="lg"
            v-ripple
            @click="store.selectElement('btn-' + (n-1))"
        >
          <v-icon size="large" color="grey-lighten-2">mdi-plus</v-icon>
        </v-card>
      </div>

    </div>
  </v-card>
</template>


<style scoped>
.hardware-board {
  width: auto;
  min-width: 500px;
  background: linear-gradient(145deg, #1e1e21, #111113) !important;
  border: 1px solid rgba(255, 255, 255, 0.05) !important;
}

/* Drehregler (Alu-Look wie auf dem Foto) */
.encoder {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  background: conic-gradient(#3f3f46, #18181b, #3f3f46);
  border: 2px solid #000;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
}

.encoder-inner {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.2s;
}

.encoder-inner.active {
  box-shadow: inset 0 0 0 3px #3b82f6, 0 0 15px rgba(59, 130, 246, 0.5);
}

.encoder-notch {
  position: absolute;
  top: 6px;
  width: 3px;
  height: 10px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 2px;
}

/* Button Grid (4 Spalten, 2 Zeilen) */
.button-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 24px;
}

.stream-btn {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid transparent !important;
  transition: all 0.2s ease;
  background: #1c1c1f !important;
}

.stream-btn.selected {
  border-color: #3b82f6 !important;
  background-color: #2d2d32 !important;
}

/* Hilfsklassen */
.gap-8 { gap: 32px; }
</style>


<script setup lang="ts">
import OledDisplay from './OledDisplay.vue';
import { useStreamDeckStore } from '@/stores/streamdeck';

const store = useStreamDeckStore();
</script>
