<template>
  <div class="mt-1">
    <div class="d-flex justify-space-between align-center mb-1">
      <div class="text-body-2 text-grey">Programm</div>
      <!-- Refresh Button triggert das Event an die Hauptkomponente -->
      <v-btn
          icon="mdi-refresh"
          variant="text"
          size="x-small"
          color="grey"
          @click="onRefresh"
      ></v-btn>
    </div>

    <v-autocomplete
        :model-value="processName"
        :items="processes"
        variant="underlined"
        density="compact"
        hide-details
        placeholder="Prozess wählen..."
        class="text-white"
        @update:model-value="onProcessUpdate"
    ></v-autocomplete>
  </div>
</template>

<script setup lang="ts">
// --- PROPS ---
const props = defineProps<{
  processName: string | undefined;
  processes: string[];
}>();

// --- EMITS ---
const emit = defineEmits<{
  (e: 'update:processName', value: string): void;
  (e: 'refresh'): void;
}>();

// --- METHODS ---
const onProcessUpdate = (val: string) => {
  emit('update:processName', val);
};

const onRefresh = () => {
  emit('refresh');
};
</script>