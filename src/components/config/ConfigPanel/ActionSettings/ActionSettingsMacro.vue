<template>
  <div class="d-flex align-center justify-space-between mt-1">
    <div class="text-body-2 text-grey">Eigene Tastenkombination</div>

    <div class="d-flex align-center gap-2">
      <!-- Makro Anzeige: Zeigt entweder den Live-Input, das gespeicherte Makro oder Fallbacks -->
      <div
          class="macro-display text-caption px-2 py-1 rounded"
          :class="{ 'is-recording': isRecording, 'has-value': !isRecording && actionKey }"
      >
        {{ isRecording ? (localDisplayKey || 'Taste drücken...') : (actionKey || 'Keine Taste') }}
      </div>

      <!-- REC Button -->
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
import { ref, onUnmounted } from 'vue';

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
const localDisplayKey = ref(''); // Zeigt die Tasten live an, ohne gleich ins Backend zu speichern
let currentKeys: string[] = [];

// --- METHODS ---
const toggleRecording = () => {
  isRecording.value = !isRecording.value;

  if (isRecording.value) {
    currentKeys = [];
    localDisplayKey.value = ''; // Anzeige leeren für den neuen Versuch
    window.addEventListener('keydown', handleKeyDown);
  } else {
    window.removeEventListener('keydown', handleKeyDown);

    // Erst beim STOPPEN der Aufnahme wird der neue Key an den Store/Backend gesendet
    if (currentKeys.length > 0) {
      emit('update:actionKey', currentKeys.join(' + '));
    }
  }
};

const handleKeyDown = (e: KeyboardEvent) => {
  e.preventDefault(); // Verhindert Browser-Shortcuts

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
    // Lokale Anzeige sofort updaten, damit der Nutzer sieht, was er drückt
    localDisplayKey.value = parts.join(' + ');
  }
};

// Wichtig: EventListener aufräumen, wenn die Komponente zerstört wird
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<style scoped>
.gap-2 {
  gap: 8px;
}

.macro-display {
  background-color: transparent;
  border: 1px dashed #52525b; /* zinc-600 */
  min-width: 140px;
  text-align: center;
  transition: all 0.2s ease;
  color: #a1a1aa;
}

.macro-display.is-recording {
  border-color: #ef4444; /* error color */
  color: #ef4444;
  background-color: rgba(239, 68, 68, 0.1);
}

.macro-display.has-value {
  border-style: solid;
  border-color: #6366f1; /* primary color */
  color: #6366f1;
}
</style>