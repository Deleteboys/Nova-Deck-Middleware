<template>
  <div class="d-flex align-center justify-space-between mt-1 flex-wrap gap-y-2">
    <div class="text-body-2 text-grey mr-2">Eigene Tastenkombination</div>

    <div class="d-flex align-center gap-2 flex-grow-1 justify-end">
      <div
          class="macro-display text-caption px-2 py-1 rounded"
          :class="{ 'is-recording': isRecording, 'has-value': !isRecording && actionKey }"
      >
        {{ isRecording ? (localDisplayKey || 'Taste drücken...') : (actionKey || 'Keine Taste') }}
      </div>

      <v-menu max-height="300">
        <template v-slot:activator="{ props }">
          <v-btn
              v-bind="props"
              size="small"
              variant="outlined"
              color="grey"
              class="text-none px-2"
              :disabled="isRecording"
              style="min-width: 65px;"
          >
            Presets
          </v-btn>
        </template>
        <v-list density="compact">
          <template v-for="(preset, index) in presets" :key="index">
            <v-list-subheader v-if="preset.header">{{ preset.header }}</v-list-subheader>
            <v-list-item v-else @click="selectPreset(preset.value!)">
              <v-list-item-title class="text-caption">{{ preset.label }}</v-list-item-title>
            </v-list-item>
          </template>
        </v-list>
      </v-menu>

      <v-btn
          size="small"
          :color="isRecording ? 'error' : 'primary'"
          :variant="isRecording ? 'flat' : 'tonal'"
          class="text-none font-weight-bold"
          style="min-width: 60px;"
          @click="toggleRecording"
      >
        {{ isRecording ? 'STOP' : 'REC' }}
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onUnmounted } from 'vue';
import { usePlatform } from '@/composables/usePlatform';

// --- PROPS ---
defineProps<{
  actionKey: string | undefined;
}>();

// --- EMITS ---
const emit = defineEmits<{
  (e: 'update:actionKey', value: string): void;
}>();

// --- STATE ---
const isRecording = ref(false);
const localDisplayKey = ref('');
let currentKeys: string[] = [];

// --- PRESETS ---
// Typisierung für die gemischte Liste (Header & Items)
type PresetItem = { header?: string; label?: string; value?: string };

const { isLinux } = usePlatform();

// 'Win + L' und 'Ctrl + Shift + Esc' behandelt das Backend nativ (siehe
// platform::*::shell). Alle anderen Presets gehen als simulierte Tasten raus –
// unter Linux ist 'Win' die Super-Taste, die Aktion hängt dort also an den
// Tastenkürzeln des Desktops.
const windowsPresets: PresetItem[] = [
  { header: 'Windows Shortcuts' },
  { label: 'Snipping Tool (Win+Shift+S)', value: 'Win + Shift + S' },
  { label: 'Task-Manager (Ctrl+Shift+Esc)', value: 'Ctrl + Shift + Esc' },
  { label: 'Desktop anzeigen (Win+D)', value: 'Win + D' },
  { label: 'PC sperren (Win+L)', value: 'Win + L' },
  { label: 'Zwischenablage (Win+V)', value: 'Win + V' },
  { label: 'Emoji-Panel (Win+.)', value: 'Win + .' },
];

const linuxPresets: PresetItem[] = [
  { header: 'Desktop-Shortcuts' },
  { label: 'Bildschirm sperren', value: 'Win + L' },
  { label: 'Systemmonitor', value: 'Ctrl + Shift + Esc' },
  { label: 'Anwendungsmenü (Super)', value: 'Win' },
  { label: 'Fenster schließen (Super+Q)', value: 'Win + Q' },
  { label: 'Fenster umschalten (Alt+Tab)', value: 'Alt + TAB' },
  { label: 'Terminal (Super+Enter)', value: 'Win + ENTER' },
];

const specialKeys: PresetItem[] = [
  { header: 'Spezialtasten' },
  { label: 'Drucken (Print Screen)', value: 'PrintScreen' },
  { label: 'Rollen (Scroll Lock)', value: 'ScrollLock' },
  { label: 'Pause / Untbr', value: 'Pause' },
  { label: 'Einfügen (Insert)', value: 'Insert' },
  { label: 'Kontextmenü (Menu-Taste)', value: 'ContextMenu' },
];

const presets = computed<PresetItem[]>(() => [
  ...(isLinux.value ? linuxPresets : windowsPresets),
  ...specialKeys,
]);

// --- METHODS ---
const selectPreset = (presetValue: string) => {
  if (isRecording.value) {
    stopRecording();
  }
  emit('update:actionKey', presetValue);
};

const stopRecording = () => {
  if (!isRecording.value) return;

  isRecording.value = false;
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('blur', handleBlur);

  if (currentKeys.length > 0) {
    emit('update:actionKey', currentKeys.join(' + '));
  }
};

const toggleRecording = () => {
  if (isRecording.value) {
    stopRecording();
  } else {
    isRecording.value = true;
    currentKeys = [];
    localDisplayKey.value = '';

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('blur', handleBlur);
  }
};

const handleBlur = () => {
  stopRecording();
};

const handleKeyDown = (e: KeyboardEvent) => {
  e.preventDefault();

  const parts = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');
  if (e.metaKey) parts.push('Win');

  const key = e.key.toUpperCase();
  if (!['CONTROL', 'SHIFT', 'ALT', 'META'].includes(key)) {
    parts.push(key === ' ' ? 'SPACE' : key);
  }

  if (parts.length > 0) {
    currentKeys = parts;
    localDisplayKey.value = parts.join(' + ');
  }
};

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('blur', handleBlur);
});
</script>

<style scoped>
.gap-y-2 {
  row-gap: 8px;
}

.gap-2 {
  gap: 8px;
}

.macro-display {
  background-color: transparent;
  border: 1px dashed #52525b;
  /* flex: 1 erlaubt der Anzeige, sich dem Platz anzupassen */
  flex: 1;
  min-width: 100px;
  max-width: 200px;
  text-align: center;
  transition: all 0.2s ease;
  color: #a1a1aa;
}

.macro-display.is-recording {
  border-color: #ef4444;
  color: #ef4444;
  background-color: rgba(239, 68, 68, 0.1);
}

.macro-display.has-value {
  border-style: solid;
  border-color: #6366f1;
  color: #6366f1;
}
</style>