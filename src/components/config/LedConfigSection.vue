<template>
  <div class="led-config pa-4 d-flex flex-column fill-height">

    <div class="mb-5">
      <div class="d-flex align-center text-caption text-primary uppercase tracking-widest font-weight-bold mb-1">
        <v-icon size="small" class="mr-2">mdi-led-variant</v-icon>
        Global LED Effects
      </div>
    </div>

    <v-divider class="mb-5 border-opacity-25" color="white"></v-divider>

    <div class="d-flex justify-space-between align-center mb-4">
      <div class="text-body-2 text-grey font-weight-medium">Aktiver Effekt</div>

      <v-select
          v-model="store.ledConfig.effect"
          :items="EFFECT_LIST"
          variant="plain"
          density="compact"
          hide-details
          class="compact-effect-select"
      ></v-select>
    </div>

    <div class="d-flex flex-column mb-8">
      <v-card color="#18181b" variant="flat" class="border border-zinc-800 rounded-lg overflow-hidden">

        <div
            v-for="(param, index) in EFFECT_PARAMS[store.ledConfig.effect]"
            :key="param"
            class="px-3 py-3"
            :class="{ 'border-b border-zinc-700': index !== EFFECT_PARAMS[store.ledConfig.effect].length - 1 }"
        >

          <template v-if="param === 'color'">
            <div class="text-body-2 text-grey mb-3">Farbe wählen</div>
            <v-color-picker
                v-model="store.ledConfig.color"
                hide-inputs flat width="100%" canvas-height="80"
                class="bg-transparent"
                elevation="0"
            ></v-color-picker>
          </template>

          <template v-else-if="param === 'reverse'">
            <div class="d-flex justify-space-between align-center">
              <div class="text-body-2 text-grey">Richtung umkehren</div>
              <v-switch
                  v-model="store.ledConfig[param]"
                  color="primary"
                  hide-details
                  density="compact"
                  inset
                  style="flex: 0 0 auto;"
              ></v-switch>
            </div>
          </template>

          <template v-else>
            <div class="d-flex justify-space-between align-center mb-1">
              <div class="text-body-2 text-grey text-capitalize">{{ param.replace('_', ' ') }}</div>

              <span
                  v-if="editingParam !== param"
                  class="text-body-2 text-white font-weight-bold edit-trigger"
                  title="Klicken zur direkten Eingabe"
                  @click="startEditing(param)"
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
                track-color="zinc-700"
                density="compact"
            ></v-slider>
          </template>
        </div>
      </v-card>
    </div>

    <v-fade-transition>
      <div v-if="store.hasUnsavedLedChanges" class="save-bar mt-auto">
        <v-btn
            color="primary"
            prepend-icon="mdi-cloud-upload"
            block
            rounded="lg"
            elevation="12"
            height="48"
            class="text-none font-weight-medium"
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
/* Angepasstes Effekt Dropdown */
.compact-effect-select {
  max-width: 150px;
}
.compact-effect-select :deep(.v-field__input) {
  font-size: 0.875rem !important;
  text-align: right;
  color: #6366f1 !important; /* primary */
  padding-top: 0 !important;
  padding-bottom: 0 !important;
  min-height: 28px !important;
}
.compact-effect-select :deep(.v-field__append-inner) {
  padding-top: 0 !important;
  align-items: center;
}

/* V-Color-Picker Tweaks für Flat-Design */
:deep(.v-color-picker) {
  background: transparent !important;
}
:deep(.v-color-picker__canvas) {
  border-radius: 8px;
  overflow: hidden;
}

.save-bar {
  position: sticky;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 20px 0 0 0;
  background: linear-gradient(to top, #18181b 80%, transparent);
  z-index: 10;
}

/* --- INLINE EDITING STYLES (Identisch zum Config-Panel) --- */
.edit-trigger {
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.03);
  transition: all 0.2s;
}

.edit-trigger:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #6366f1 !important;
}

.inline-input-wrapper {
  width: 55px;
  margin-top: -6px;
}

.inline-input-wrapper :deep(input) {
  text-align: right;
  font-size: 0.875rem !important;
  font-weight: bold;
  color: white !important;
  padding-bottom: 2px !important;
}

.inline-input-wrapper :deep(input[type="number"]::-webkit-outer-spin-button),
.inline-input-wrapper :deep(input[type="number"]::-webkit-inner-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
</style>