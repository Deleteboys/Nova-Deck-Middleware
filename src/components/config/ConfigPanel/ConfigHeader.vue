<template>
  <div class="mb-5">
    <div class="text-caption text-primary uppercase tracking-widest font-weight-bold mb-1">
      {{ isOled ? 'OLED Panel' : elementId }}
    </div>

    <v-text-field
        v-if="!isOled"
        :model-value="label"
        @update:model-value="$emit('update:label', $event)"
        placeholder="Button Name eingeben..."
        variant="plain"
        density="compact"
        hide-details
        class="seamless-title-input text-white"
        @input="$emit('save')"
    ></v-text-field>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  isOled: boolean;
  elementId: string;
  label: string;
}>();

defineEmits<{
  (e: 'update:label', value: string): void;
  (e: 'save'): void;
}>();
</script>

<style scoped>
.seamless-title-input :deep(input) {
  font-size: 1.15rem !important;
  font-weight: 700 !important;
  padding: 4px 0 !important;
  opacity: 0.9;
  line-height: 1.3 !important;
  transition: all 0.2s ease;
}
.seamless-title-input :deep(input:focus) {
  opacity: 1;
}
</style>