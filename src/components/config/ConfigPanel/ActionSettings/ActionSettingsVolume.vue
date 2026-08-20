<template>
  <div>
    <div class="d-flex justify-space-between align-center mb-1">
      <div class="text-body-2 text-grey">Intervall</div>

      <span
          v-if="!isEditing"
          class="text-body-2 text-white font-weight-bold edit-trigger"
          title="Klicken zur direkten Eingabe"
          @click="startEditing"
      >
        {{ step > 0 ? '+' : '' }}{{ step }}%
      </span>

      <div v-else class="inline-input-wrapper">
        <v-text-field
            :model-value="step"
            type="number"
            density="compact"
            variant="underlined"
            hide-details
            autofocus
            color="primary"
            @update:model-value="onInputUpdate"
            @blur="stopEditing"
            @keyup.enter="stopEditing"
        ></v-text-field>
      </div>
    </div>

    <v-slider
        :model-value="step"
        :min="-50"
        :max="50"
        :step="1"
        hide-details
        color="primary"
        track-color="zinc-700"
        @update:model-value="onSliderUpdate"
    ></v-slider>

    <div v-if="showSnap" class="d-flex align-center justify-space-between px-1 pt-1">
      <div class="d-flex align-center">
        <span class="text-caption text-grey">Am Raster einrasten</span>
        <span class="d-inline-flex align-center ml-1 text-help">
          <v-icon icon="mdi-information-outline" size="x-small" color="grey" />
          <v-tooltip activator="parent" location="top" open-delay="250" max-width="260">
            Springt auf runde Werte, statt exakt um das Intervall zu ändern
          </v-tooltip>
        </span>
      </div>
      <v-switch
          :model-value="snap"
          density="compact"
          hide-details
          color="primary"
          class="flex-shrink-0"
          style="margin-top: 0"
          @update:model-value="onSnapUpdate"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

// --- PROPS ---
withDefaults(defineProps<{
  step: number;
  showSnap?: boolean;
  snap?: boolean;
}>(), {
  showSnap: false,
  snap: false,
});

// --- EMITS ---
const emit = defineEmits<{
  (e: 'update:step', value: number): void;
  (e: 'update:snap', value: boolean): void;
}>();

// --- STATE ---
const isEditing = ref(false);

// --- METHODS ---
const startEditing = () => {
  isEditing.value = true;
};

const stopEditing = () => {
  isEditing.value = false;
};

const onInputUpdate = (val: string | number) => {
  const numVal = Number(val);
  if (!isNaN(numVal)) {
    emit('update:step', numVal);
  }
};

const onSliderUpdate = (val: number) => {
  emit('update:step', val);
};

const onSnapUpdate = (val: boolean | null) => {
  emit('update:snap', !!val);
};
</script>

<style scoped>
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
</style>