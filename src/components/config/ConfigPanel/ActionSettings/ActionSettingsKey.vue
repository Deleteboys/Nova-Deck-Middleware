<template>
  <div class="d-flex align-center justify-space-between">
    <div class="text-body-2 text-grey">Tastenkombination</div>
    <v-select
        :model-value="actionKey"
        :items="keysList"
        variant="underlined"
        density="compact"
        hide-details
        class="compact-key-select"
        @update:model-value="onKeyUpdate"
    ></v-select>
  </div>
</template>

<script setup lang="ts">
// --- PROPS ---
const props = defineProps<{
  actionKey: string | undefined;
  keysList: string[];
}>();

// --- EMITS ---
const emit = defineEmits<{
  (e: 'update:actionKey', value: string): void;
}>();

// --- METHODS ---
const onKeyUpdate = (val: string) => {
  emit('update:actionKey', val);
};
</script>

<style scoped>
.compact-key-select {
  max-width: 140px;
}

.compact-key-select :deep(.v-field__input) {
  font-size: 0.875rem !important;
  text-align: right;
  color: #6366f1 !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}
</style>