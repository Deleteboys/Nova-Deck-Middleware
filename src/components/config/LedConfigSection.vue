<template>
  <div class="led-config pb-12">
    <h3 class="mb-6 d-flex align-center text-uppercase text-caption font-weight-bold text-grey">
      <v-icon size="small" class="mr-2">mdi-led-variant</v-icon> Global LED Effects
    </h3>

    <v-select
        v-model="store.ledConfig.effect"
        :items="EFFECT_LIST"
        label="Effekt"
        variant="filled"
        density="compact"
        rounded="lg"
        class="mb-4"
    ></v-select>

    <v-divider class="mb-6"></v-divider>

    <div class="controls-container">
      <div v-for="param in EFFECT_PARAMS[store.ledConfig.effect]" :key="param" class="control-group mb-4">

        <template v-if="param === 'color'">
          <label class="text-caption text-grey mb-2 d-block text-uppercase font-weight-medium">Farbe</label>
          <v-color-picker
              v-model="store.ledConfig.color"
              hide-inputs flat width="100%" canvas-height="80"
              class="bg-transparent border rounded-lg"
          ></v-color-picker>
        </template>

        <template v-else-if="param === 'reverse'">
          <div class="d-flex justify-space-between align-center mb-1">
            <label class="text-caption text-grey text-uppercase">Richtung Umkehren</label>
          </div>
          <v-switch
              v-model="store.ledConfig[param]"
              color="primary"
              hide-details
              density="compact"
              inset
          ></v-switch>
        </template>

        <template v-else>
          <div class="d-flex justify-space-between align-center mb-1">
            <label class="text-caption text-grey text-uppercase">{{ param.replace('_', ' ') }}</label>

            <span
                v-if="editingParam !== param"
                class="text-caption font-weight-bold text-primary edit-trigger"
                title="Doppelklick zur direkten Eingabe"
                @dblclick="startEditing(param)"
            >
              {{ Math.round(store.ledConfig[param]) }}
            </span>

            <div v-else class="inline-input-wrapper">
              <v-text-field
                  v-model.number="store.ledConfig[param]"
                  type="number"
                  :min="0"
                  :max="param === 'hue' ? 360 : 255"
                  density="compact"
                  variant="underlined"
                  hide-details
                  autofocus
                  color="primary"
                  @blur="stopEditing(param)"
                  @keyup.enter="stopEditing(param)"
              ></v-text-field>
            </div>
          </div>

          <v-slider
              v-model="store.ledConfig[param]"
              :min="0"
              :max="param === 'hue' ? 360 : 255"
              step="1"
              hide-details
              color="primary"
              density="compact"
          ></v-slider>
        </template>
      </div>
    </div>

    <v-fade-transition>
      <div v-if="store.hasUnsavedLedChanges" class="save-bar">
        <v-btn
            color="primary"
            prepend-icon="mdi-cloud-upload"
            block
            rounded="lg"
            elevation="12"
            height="48"
            @click="store.saveLedSettings"
        >
          An Streamdeck senden
        </v-btn>
      </div>
    </v-fade-transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';

const store = useStreamDeckStore();

// --- State und Funktionen für das Inline-Editing ---
const editingParam = ref<string | null>(null);

const startEditing = (param: string) => {
  editingParam.value = param;
};

const stopEditing = (param: string) => {
  // Sicherheits-Check: Begrenzt die Eingabe auf den korrekten Maximalwert
  const max = param === 'hue' ? 360 : 255;
  if (store.ledConfig[param] > max) store.ledConfig[param] = max;
  if (store.ledConfig[param] < 0) store.ledConfig[param] = 0;

  editingParam.value = null;
};
// ---------------------------------------------------

const EFFECT_PARAMS: Record<string, string[]> = {
  Solid: ['color', 'brightness'],
  Blink: ['color', 'brightness', 'speed'],
  Rainbow: ['brightness', 'speed', 'saturation', 'reverse'],
  Breathing: ['color', 'brightness', 'speed'],
  Chase: ['color', 'brightness', 'speed', 'size', 'reverse'],
  Comet: ['color', 'brightness', 'speed', 'tail', 'reverse'],
  Sparkle: ['color', 'brightness', 'speed', 'density'],
  Aurora: ['brightness', 'speed', 'reverse'],
  ColorOrbit: ['hue', 'hue_shift', 'saturation', 'brightness', 'speed', 'reverse'],
  Astolfo: ['brightness', 'speed', 'saturation', 'spread', 'reverse']
};

const EFFECT_LIST = Object.keys(EFFECT_PARAMS);
</script>

<style scoped>
.control-group {
  background: rgba(255, 255, 255, 0.03);
  padding: 12px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.text-primary { color: #6366f1 !important; }

.save-bar {
  position: sticky;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 20px 12px;
  /* Sanfter Verlauf nach oben, damit der Content unter dem Button verschwindet */
  background: linear-gradient(to top, #18181b 70%, transparent);
  margin-top: 20px;
}

/* --- Styles für das Inline-Editing --- */
.edit-trigger {
  cursor: pointer;
  padding: 0 4px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.edit-trigger:hover {
  background: rgba(99, 102, 241, 0.1);
}

.inline-input-wrapper {
  width: 60px;
  margin-top: -8px;
}

.inline-input-wrapper :deep(input[type="number"]::-webkit-outer-spin-button),
.inline-input-wrapper :deep(input[type="number"]::-webkit-inner-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
.inline-input-wrapper :deep(input[type="number"]) {
  -moz-appearance: textfield;
  text-align: right;
}
</style>