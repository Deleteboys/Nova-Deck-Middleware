<template>
  <div class="h-100 w-100 d-flex flex-column pa-6 bg-zinc-950 overflow-hidden custom-scrollbar">

    <div class="flex-shrink-0 mb-6 d-flex align-start flex-wrap">
      <div>
        <h2 class="text-h4 font-weight-bold text-white mb-1">Developer Console</h2>
        <p class="text-grey mb-0">System-Konsolenausgabe und Debug-Logs.</p>
      </div>

      <v-spacer></v-spacer>

      <v-btn
          variant="tonal"
          color="grey-lighten-1"
          :prepend-icon="showDiagnostics ? 'mdi-eye-off' : 'mdi-eye'"
          class="text-none mt-2 mt-sm-0"
          size="small"
          @click="showDiagnostics = !showDiagnostics"
      >
        {{ showDiagnostics ? 'Diagnostics ausblenden' : 'Diagnostics einblenden' }}
      </v-btn>
    </div>

    <v-card
        v-show="showDiagnostics"
        class="flex-shrink-0 mb-4"
        elevation="0"
        rounded="xl"
        color="#18181b"
        border
        style="border-color: rgba(255, 255, 255, 0.05) !important;"
    >
      <div class="d-flex align-center px-4 py-2 border-b flex-wrap"
           style="border-color: rgba(255, 255, 255, 0.05) !important; background-color: rgba(0,0,0,0.2);">
        <v-icon icon="mdi-memory" color="grey" size="small" class="mr-3"></v-icon>
        <div class="text-caption text-grey text-uppercase font-weight-bold letter-spacing-1 mr-4">
          Runtime Diagnostics
        </div>

        <v-chip v-if="diagnostics" size="small" variant="tonal" color="primary" class="mr-2 mt-1 mt-sm-0">
          PID {{ diagnostics.process.pid }}
        </v-chip>
        <v-chip v-if="diagnostics" size="small" variant="tonal" color="grey" class="mt-1 mt-sm-0">
          Laufzeit {{ formatDuration(diagnostics.uptime_secs) }}
        </v-chip>

        <v-spacer></v-spacer>

        <v-switch
            v-model="autoRefreshDiagnostics"
            color="primary"
            density="compact"
            hide-details
            label="Auto"
            class="mr-3 mt-1 mt-sm-0"
        ></v-switch>

        <v-btn
            size="small"
            variant="text"
            color="grey-lighten-1"
            prepend-icon="mdi-content-copy"
            class="text-none mr-2 mt-1 mt-sm-0"
            :disabled="!diagnostics"
            @click="copyDiagnostics"
        >
          Kopieren
        </v-btn>

        <v-btn
            size="small"
            variant="text"
            color="grey-lighten-1"
            prepend-icon="mdi-refresh"
            class="text-none mt-1 mt-sm-0"
            :loading="isLoadingDiagnostics"
            @click="refreshDiagnostics"
        >
          Aktualisieren
        </v-btn>
      </div>

      <div class="pa-4 overflow-y-auto custom-scrollbar" style="max-height: 35vh;">
        <div v-if="diagnosticsError" class="text-error text-body-2">{{ diagnosticsError }}</div>
        <div v-else-if="!diagnostics" class="text-grey text-body-2">Lade Diagnosewerte...</div>
        <div v-else class="diagnostics-grid">
          <div v-for="item in diagnosticItems" :key="item.label" class="diagnostic-cell">
            <div class="text-caption text-grey">{{ item.label }}</div>
            <div class="text-body-2 text-white font-weight-bold">{{ item.value }}</div>
            <div v-if="item.hint" class="text-caption text-grey-darken-1">{{ item.hint }}</div>
          </div>
        </div>
      </div>
    </v-card>

    <v-card
        class="flex-grow-1 d-flex flex-column overflow-hidden"
        elevation="0"
        rounded="xl"
        color="#18181b"
        border
        style="border-color: rgba(255, 255, 255, 0.05) !important; min-height: 0;"
    >
      <div class="d-flex align-center px-4 py-2 border-b flex-wrap"
           style="border-color: rgba(255, 255, 255, 0.05) !important; background-color: rgba(0,0,0,0.2);">

        <v-icon icon="mdi-console" color="grey" size="small" class="mr-3"></v-icon>
        <div class="text-caption text-grey text-uppercase font-weight-bold letter-spacing-1 mr-4">
          Live Logs
        </div>

        <v-divider vertical class="mx-2 my-2" color="grey-darken-3"></v-divider>

        <v-chip-group
            v-model="activeLogLevels"
            multiple
            selected-class="text-white"
            class="ma-0 ml-2"
        >
          <v-chip
              v-for="level in logLevels"
              :key="level.value"
              :value="level.value"
              :color="level.color"
              filter
              variant="tonal"
              size="small"
              class="font-weight-bold text-caption"
          >
            {{ level.name }}
          </v-chip>
        </v-chip-group>

        <v-spacer></v-spacer>

        <v-btn
            size="small"
            variant="text"
            color="grey-lighten-1"
            prepend-icon="mdi-delete"
            class="text-none mt-1 mt-sm-0"
            @click="store.clearLogs()"
        >
          Leeren
        </v-btn>
      </div>

      <div class="flex-grow-1 overflow-y-auto pa-4 terminal-window custom-scrollbar" style="background-color: #09090b;">
        <div v-if="filteredLogs.length === 0"
             class="h-100 d-flex flex-column align-center justify-center text-grey-darken-2">
          <v-icon icon="mdi-sleep" size="x-large" class="mb-3 opacity-50"></v-icon>
          <div>
            {{
              store.debugLogs.length === 0 ? 'Warte auf Systemereignisse...' : 'Keine Logs für gewählte Filter gefunden.'
            }}
          </div>
        </div>

        <div
            v-for="(log, idx) in filteredLogs"
            :key="idx"
            class="log-entry d-flex py-1"
        >
          <span class="log-time text-grey-darken-1 mr-4">[{{ log.timestamp }}]</span>
          <span :class="getLogLevelClass(log.level)" class="log-level mr-4 font-weight-bold">
            [{{ getLogLevelName(log.level) }}]
          </span>
          <span class="log-message text-grey-lighten-1">{{ log.message }}</span>
        </div>
      </div>
    </v-card>

  </div>
</template>

<script setup lang="ts">
import {ref, computed, onMounted, onUnmounted, watch} from "vue";
import {useStreamDeckStore} from "@/stores/streamdeck";
import {getRuntimeDiagnostics, type RuntimeDiagnostics} from "@/services/streamdeckCommands";

const store = useStreamDeckStore();
const diagnostics = ref<RuntimeDiagnostics | null>(null);
const diagnosticsError = ref("");
const isLoadingDiagnostics = ref(false);
const autoRefreshDiagnostics = ref(true);
const showDiagnostics = ref(true); // Toggle-Zustand für Diagnostics
let diagnosticsInterval: number | null = null;

// --- Log Filter Logik ---

// Definition der verfügbaren Log-Level mit ihren UI-Eigenschaften
const logLevels = [
  { value: 5, name: 'ERROR', color: 'error' },
  { value: 4, name: 'WARN',  color: 'warning' },
  { value: 3, name: 'INFO',  color: 'primary' },
  { value: 2, name: 'DEBUG', color: 'grey' },
  { value: 1, name: 'TRACE', color: 'grey-darken-2' },
];

const activeLogLevels = ref<number[]>([3, 4, 5]);

const filteredLogs = computed(() => {
  return store.debugLogs.filter(log => activeLogLevels.value.includes(log.level));
});

const formatBytes = (bytes: number) => {
  const mb = bytes / 1024 / 1024;
  return `${mb.toFixed(1)} MB`;
};

const formatNumber = (value: number) => new Intl.NumberFormat("de-DE").format(value);

const formatDuration = (seconds: number) => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m ${secs}s`;
  return `${secs}s`;
};

const diagnosticItems = computed(() => {
  if (!diagnostics.value) return [];

  const d = diagnostics.value;
  return [
    {
      label: "Private RAM",
      value: formatBytes(d.process.private_usage_bytes),
      hint: "Wichtigster Wert fuer Leak-Verdacht"
    },
    {
      label: "Working Set",
      value: formatBytes(d.process.working_set_bytes),
      hint: `Peak ${formatBytes(d.process.peak_working_set_bytes)}`
    },
    {
      label: "Handles",
      value: formatNumber(d.process.handle_count),
      hint: "Steigt bei Handle-Leaks dauerhaft"
    },
    {
      label: "Page Faults",
      value: formatNumber(d.process.page_faults),
      hint: "Nur Vergleichswert"
    },
    {
      label: "Serial Buffer",
      value: `${d.serial.accumulator_len} / ${d.serial.accumulator_capacity} B`,
      hint: `Max ${d.serial.accumulator_max_len} B, Drops ${d.serial.buffer_drops}`
    },
    {
      label: "Serial Traffic",
      value: `${formatBytes(d.serial.bytes_read)} gelesen`,
      hint: `${formatNumber(d.serial.messages_decoded)} Messages, ${formatNumber(d.serial.parse_errors)} Parse-Errors`
    },
    {
      label: "Serial Events",
      value: formatNumber(d.serial.pico_events_emitted),
      hint: `${formatNumber(d.serial.host_commands_written)} Commands zum Pico`
    },
    {
      label: "Audio Monitor",
      value: `${formatNumber(d.audio.snapshots)} Snapshots`,
      hint: `${formatNumber(d.audio.ticks)} Ticks, ${formatNumber(d.audio.slot_polls)} Slot-Polls`
    },
    {
      label: "Audio Sessions",
      value: formatNumber(d.audio.sessions_enumerated),
      hint: `${formatNumber(d.audio.status_errors)} Fehler, ${formatNumber(d.audio.empty_results)} leere Ergebnisse`
    },
    {
      label: "Audio Events",
      value: formatNumber(d.audio.updates_emitted),
      hint: `${formatNumber(d.audio.pico_commands_sent)} Commands zum Pico`
    },
    {
      label: "COM",
      value: `${formatNumber(d.com.real_inits)} init / ${formatNumber(d.com.uninits)} free`,
      hint: `${formatNumber(d.com.reused_inits)} reused, ${formatNumber(d.com.changed_mode_results)} changed-mode`
    },
  ];
});

const refreshDiagnostics = async () => {
  isLoadingDiagnostics.value = true;
  diagnosticsError.value = "";

  try {
    diagnostics.value = await getRuntimeDiagnostics();
  } catch (error) {
    diagnosticsError.value = String(error);
  } finally {
    isLoadingDiagnostics.value = false;
  }
};

const copyDiagnostics = async () => {
  if (!diagnostics.value) return;

  const text = JSON.stringify(diagnostics.value, null, 2);
  await navigator.clipboard.writeText(text);
};

const startDiagnosticsTimer = () => {
  if (diagnosticsInterval !== null) return;
  diagnosticsInterval = window.setInterval(refreshDiagnostics, 5000);
};

const stopDiagnosticsTimer = () => {
  if (diagnosticsInterval === null) return;
  window.clearInterval(diagnosticsInterval);
  diagnosticsInterval = null;
};

watch(autoRefreshDiagnostics, (enabled) => {
  if (enabled) {
    startDiagnosticsTimer();
  } else {
    stopDiagnosticsTimer();
  }
});

onMounted(() => {
  refreshDiagnostics();
  if (autoRefreshDiagnostics.value) {
    startDiagnosticsTimer();
  }
});

onUnmounted(() => {
  stopDiagnosticsTimer();
});

// --- Log Formatierung ---
const getLogLevelName = (level: number) => {
  switch (level) {
    case 5: return 'ERROR';
    case 4: return 'WARN';
    case 3: return 'INFO';
    case 2: return 'DEBUG';
    case 1: return 'TRACE';
    default: return 'LOG';
  }
};

const getLogLevelClass = (level: number) => {
  switch (level) {
    case 5: return 'text-error';
    case 4: return 'text-warning';
    case 3: return 'text-primary';
    case 2: return 'text-grey';
    case 1: return 'text-grey-darken-2';
    default: return 'text-white';
  }
};
</script>

<style scoped>
.bg-zinc-950 {
  background-color: #09090b !important;
}

.terminal-window {
  font-family: 'Fira Code', 'Courier New', Courier, monospace;
  font-size: 13px;
  line-height: 1.6;
}

.log-entry {
  word-break: break-all;
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
}

.log-entry:hover {
  background-color: rgba(255, 255, 255, 0.03);
}

.log-time {
  user-select: none;
  min-width: 85px;
}

.log-level {
  user-select: none;
  min-width: 70px;
  white-space: pre; /* Verhindert, dass das Leerzeichen bei WARN / INFO verschluckt wird */
}

.diagnostics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.diagnostic-cell {
  min-height: 78px;
  padding: 10px 12px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.025);
}

.letter-spacing-1 {
  letter-spacing: 1px;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #27272a;
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #3f3f46;
}
</style>