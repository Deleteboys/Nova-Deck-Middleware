<template>
  <div class="mb-5">
    <div class="text-caption text-primary uppercase tracking-widest font-weight-bold mb-1">
      {{ isOled ? 'OLED Panel' : elementId }}
    </div>

    <!-- Label Input -->
    <v-text-field
        v-if="!isOled"
        :model-value="label"
        @update:model-value="$emit('update:label', $event)"
        placeholder="Button Name eingeben..."
        variant="plain"
        density="compact"
        hide-details
        class="seamless-title-input text-white mb-4"
    ></v-text-field>

    <!-- Icon Textfeld -->
    <div v-if="!isOled && elementId.startsWith('btn-')">
      <div class="d-flex justify-space-between align-center mb-1">
        <div class="text-body-2 text-grey">Icon Name (MDI)</div>

        <div
            @click="openIconLink"
            class="text-caption text-primary cursor-pointer hover-link d-flex align-center"
            title="Material Design Icons durchsuchen"
        >
          Icons suchen <v-icon size="x-small" class="ml-1">mdi-open-in-new</v-icon>
        </div>
      </div>

      <!-- v-model auf den lokalen State geändert und Event-Handler angepasst -->
      <v-text-field
          v-model="localIcon"
          @update:model-value="onIconInput"
          placeholder="z.B. mdi-spotify"
          variant="outlined"
          density="compact"
          hide-details
          bg-color="#18181b"
      >
        <template v-slot:prepend-inner>
          <v-icon :color="localIcon ? 'primary' : 'grey-darken-1'">
            {{ localIcon || 'mdi-image-search' }}
          </v-icon>
        </template>
      </v-text-field>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue';
import { open } from '@tauri-apps/plugin-shell';

const props = defineProps<{
  isOled: boolean;
  elementId: string;
  label: string;
  icon?: string;
}>();

const emit = defineEmits<{
  (e: 'update:label', value: string): void;
  (e: 'update:icon', value: string): void;
}>();

const openIconLink = async () => {
  await open('https://pictogrammers.com/library/mdi/');
};

const localIcon = ref(props.icon || '');

watch(() => props.icon, (newVal) => {
  localIcon.value = newVal || '';
});

let timeoutId: ReturnType<typeof setTimeout> | null = null;

const onIconInput = (value: string) => {
  localIcon.value = value;

  if (timeoutId) clearTimeout(timeoutId);

  timeoutId = setTimeout(() => {
    emit('update:icon', value);
  }, 500);
};

onUnmounted(() => {
  if (timeoutId) clearTimeout(timeoutId);
});
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

.cursor-pointer { cursor: pointer; }
.hover-link:hover { text-decoration: underline !important; }
</style>