<template>
  <div class="d-flex flex-column gap-2 mt-1">
    <div class="d-flex align-center justify-space-between mb-1">
      <div class="text-body-2 text-grey">Audiogeräte</div>
      <v-btn
          icon="mdi-refresh"
          variant="text"
          size="x-small"
          color="grey"
          @click="onRefresh"
      ></v-btn>
    </div>

    <!-- Dropdown für Gerät 1 -->
    <div class="d-flex align-center justify-space-between">
      <div class="text-caption text-grey">Gerät 1</div>
      <v-select
          :model-value="device1"
          :items="audioDevices"
          item-title="name"
          item-value="name"
          variant="underlined"
          density="compact"
          hide-details
          class="compact-key-select"
          @update:model-value="onDevice1Update"
      ></v-select>
    </div>

    <!-- Dropdown für Gerät 2 -->
    <div class="d-flex align-center justify-space-between">
      <div class="text-caption text-grey">Gerät 2</div>
      <v-select
          :model-value="device2"
          :items="audioDevices"
          item-title="name"
          item-value="name"
          variant="underlined"
          density="compact"
          hide-details
          class="compact-key-select"
          @update:model-value="onDevice2Update"
      ></v-select>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- TYPES ---
export interface AudioDeviceInfo {
  name: string;
  // Falls dein Backend noch mehr Infos wie IDs liefert, kannst du sie hier ergänzen
}

// --- PROPS ---
const props = defineProps<{
  device1: string | undefined;
  device2: string | undefined;
  audioDevices: AudioDeviceInfo[];
}>();

// --- EMITS ---
const emit = defineEmits<{
  (e: 'update:device1', value: string): void;
  (e: 'update:device2', value: string): void;
  (e: 'refresh'): void;
}>();

// --- METHODS ---
const onDevice1Update = (val: string) => {
  emit('update:device1', val);
};

const onDevice2Update = (val: string) => {
  emit('update:device2', val);
};

const onRefresh = () => {
  emit('refresh');
};
</script>

<style scoped>
.gap-2 {
  gap: 8px;
}

.compact-key-select {
  max-width: 180px;
}

.compact-key-select :deep(.v-field__input) {
  font-size: 0.875rem !important;
  text-align: right;
  color: #6366f1 !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}
</style>