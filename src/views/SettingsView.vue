<template>
  <div class="pa-4 d-flex flex-column fill-height overflow-y-auto custom-scrollbar">
    <div class="mx-auto w-100" style="max-width: 600px; margin-top: 20px;">

      <div class="mb-4">
        <div class="d-flex align-center text-caption text-primary uppercase tracking-widest font-weight-bold mb-1">
          <v-icon size="small" class="mr-2">mdi-microchip</v-icon>
          Firmware
        </div>
      </div>

      <v-divider class="mb-6 border-opacity-25" color="white"></v-divider>

      <v-card color="#18181b" variant="flat" class="border border-zinc-800 rounded-lg overflow-hidden mb-4">
        <div class="d-flex justify-space-between align-center px-4 py-3">
          <div class="text-body-2 text-grey">Installierte Version</div>
          <div class="d-flex align-center">
            <v-btn
                variant="tonal"
                color="primary"
                size="small"
                :loading="isChecking"
                prepend-icon="mdi-refresh"
                class="text-none mr-3"
                @click="manualUpdateCheck"
            >
              Prüfen
            </v-btn>
            <div class="text-caption font-weight-bold text-zinc-500 font-monospace bg-zinc-900 px-2 py-1 rounded">
              {{ currentVersion }}
            </div>
          </div>
        </div>
      </v-card>

      <v-expand-transition>
        <v-card
            v-if="updateAvailable || updatePhase > 1"
            color="#18181b"
            variant="flat"
            class="border border-zinc-800 rounded-lg overflow-hidden mb-10"
        >
          <div class="px-4 py-3 border-b border-zinc-700 bg-zinc-900 d-flex align-center justify-space-between">
            <div class="d-flex align-center">
              <v-icon size="small" color="primary" class="mr-2">mdi-tray-arrow-down</v-icon>
              <span class="text-body-2 font-weight-medium text-white">Firmware Update</span>
            </div>
            <v-chip size="small" color="primary" variant="flat" v-if="updatePhase === 1">Verfügbar</v-chip>
          </div>

          <div class="pa-5">
            <v-window v-model="updatePhase" :touch="false">
              <v-window-item :value="1">
                <div class="d-flex align-center justify-space-between mb-5 pa-4 bg-zinc-900 rounded-lg border border-zinc-800">
                  <div class="text-center flex-1-1-0">
                    <div class="text-caption text-grey mb-1">Aktuell</div>
                    <div class="text-caption font-weight-bold text-zinc-500 font-monospace">{{ currentVersion }}</div>
                  </div>
                  <v-icon color="grey-darken-2" class="mx-2">mdi-arrow-right</v-icon>
                  <div class="text-center flex-1-1-0">
                    <div class="text-caption text-grey mb-1">Neu</div>
                    <div class="text-caption font-weight-bold text-primary font-monospace">{{ newVersion }}</div>
                  </div>
                </div>

                <v-btn
                    color="primary"
                    block
                    variant="flat"
                    class="text-none font-weight-medium"
                    @click="startUpdate"
                >
                  Jetzt installieren
                </v-btn>
              </v-window-item>

              <v-window-item :value="2">
                <div class="text-center py-4">
                  <v-progress-circular indeterminate color="primary" size="40" width="3" class="mb-4"></v-progress-circular>
                  <div class="text-body-2 text-white font-weight-medium mb-4">{{ statusMessage }}</div>
                  <v-progress-linear v-model="updateProgress" color="primary" height="4" rounded bg-color="zinc-800"></v-progress-linear>
                </div>
              </v-window-item>

              <v-window-item :value="3">
                <div class="text-center py-4">
                  <v-icon color="success" size="48" class="mb-3">mdi-check-circle-outline</v-icon>
                  <div class="text-body-2 text-white font-weight-medium mb-1">Update erfolgreich</div>
                  <div class="text-caption text-grey mb-5">Der Pico wurde neu gestartet und ist bereit.</div>
                  <v-btn variant="tonal" color="primary" block class="text-none" @click="resetUpdate">Schließen</v-btn>
                </div>
              </v-window-item>
            </v-window>
          </div>
        </v-card>
      </v-expand-transition>


      <div class="mb-4">
        <div class="d-flex align-center text-caption text-primary uppercase tracking-widest font-weight-bold mb-1">
          <v-icon size="small" class="mr-2">mdi-cog-outline</v-icon>
          System Einstellungen
        </div>
      </div>

      <v-divider class="mb-6 border-opacity-25" color="white"></v-divider>

      <v-card color="#18181b" variant="flat" class="border border-zinc-800 rounded-lg overflow-hidden mb-8">
        <div class="d-flex justify-space-between align-center px-4 py-3 border-b border-zinc-700">
          <div class="text-body-2 text-grey">Auto-Start mit Windows</div>
          <v-switch color="primary" hide-details density="compact" inset style="flex: 0 0 auto;"></v-switch>
        </div>

        <div class="d-flex justify-space-between align-center px-4 py-3 border-b border-zinc-700">
          <div class="text-body-2 text-grey">Vibration standardmäßig an</div>
          <v-switch color="primary" hide-details density="compact" inset style="flex: 0 0 auto;"></v-switch>
        </div>

        <div class="d-flex justify-space-between align-center px-4 py-3">
          <div class="text-body-2 text-grey">Middleware Version</div>
          <div class="d-flex align-center">
            <v-btn
                variant="tonal"
                color="primary"
                size="small"
                :loading="isCheckingMiddleware"
                prepend-icon="mdi-refresh"
                class="text-none mr-3"
                @click="manualMiddlewareCheck"
            >
              Prüfen
            </v-btn>
            <div class="text-caption font-weight-bold text-zinc-500 font-monospace bg-zinc-900 px-2 py-1 rounded">
              v{{ appVersion }}
            </div>
          </div>
        </div>
      </v-card>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  startBootloader,
  checkFirmwareUpdate,
  downloadAndFlashFirmware,
  requestFirmwareVersion
} from "@/services/streamdeckCommands";
import { getVersion } from "@tauri-apps/api/app";
import {emit, listen} from "@tauri-apps/api/event";

// UI State Firmware
const updatePhase = ref(1);
const updateProgress = ref(0);
const statusMessage = ref("");
const isChecking = ref(false);

// UI State Middleware
const isCheckingMiddleware = ref(false);

// Versioning
const currentVersion = ref("Lade...");
const newVersion = ref("");
const downloadUrl = ref("");
const updateAvailable = ref(false);
const appVersion = ref("0.0.0");

let unlistenStatus: (() => void) | null = null;
let unlistenVersion: (() => void) | null = null;

onMounted(async () => {
  try {
    appVersion.value = await getVersion();

    // Listener für Flash-Status (Events vom Rust-Backend)
    unlistenStatus = await listen<string>("fw-status", (event) => {
      statusMessage.value = event.payload;
      if (event.payload.toLowerCase().includes("herunterladen")) updateProgress.value = 30;
      if (event.payload.toLowerCase().includes("warte")) updateProgress.value = 60;
      if (event.payload.toLowerCase().includes("kopiere")) updateProgress.value = 90;
    });

    // Listener für die Version vom Pico
    unlistenVersion = await listen<string>("pico-version", async (event) => {
      currentVersion.value = "v" + event.payload;
      console.log("Version empfangen: " + event.payload);

      // Automatische Prüfung im Hintergrund beim Start
      performUpdateCheck();
    });

    // Aktiv die Version beim Pico anfragen
    await requestFirmwareVersion();

  } catch (e) {
    console.error("Fehler beim Initialisieren der Settings-Ansicht", e);
  }
});

onUnmounted(() => {
  if (unlistenStatus) unlistenStatus();
  if (unlistenVersion) unlistenVersion();
});

// --- Firmware Logic ---
const performUpdateCheck = async () => {
  const updateInfo = await checkFirmwareUpdate();
  if (updateInfo && updateInfo.version !== currentVersion.value) {
    newVersion.value = updateInfo.version;
    downloadUrl.value = updateInfo.download_url;
    updateAvailable.value = true;
  } else {
    updateAvailable.value = false;
  }
};

const manualUpdateCheck = async () => {
  isChecking.value = true;
  await performUpdateCheck();
  setTimeout(() => {
    isChecking.value = false;
  }, 800);
};

const startUpdate = async () => {
  if (!downloadUrl.value) return;

  updatePhase.value = 2;
  updateProgress.value = 10;
  statusMessage.value = "Starte Bootloader...";

  try {
    await startBootloader();
    await downloadAndFlashFirmware(downloadUrl.value);

    updateProgress.value = 100;
    statusMessage.value = "Update abgeschlossen!";

    setTimeout(() => {
      updatePhase.value = 3;
      currentVersion.value = newVersion.value;
      updateAvailable.value = false;
    }, 1200);

  } catch (error) {
    updatePhase.value = 1;
    alert(`Firmware-Update fehlgeschlagen: ${String(error)}`);
  }
};

const resetUpdate = () => {
  updatePhase.value = 1;
  updateProgress.value = 0;
};

// --- Middleware Logic ---
const manualMiddlewareCheck = async () => {
  isCheckingMiddleware.value = true;

  try {
    // Feuert ein Event an das gesamte System (wird vom AppUpdater aufgefangen)
    await emit("trigger-update-check", { manual: true });
  } catch (error) {
    console.error("Fehler beim Senden des Update-Events:", error);
  } finally {
    // Kurzes Timeout, damit der Button-Ladekreis nicht sofort verschwindet
    setTimeout(() => {
      isCheckingMiddleware.value = false;
    }, 800);
  }

  setTimeout(() => {
    isCheckingMiddleware.value = false;
  }, 1000);
};
</script>

<style scoped>
.uppercase {
  text-transform: uppercase;
}

.tracking-widest {
  letter-spacing: 0.1em !important;
}

.font-monospace {
  font-family: 'Courier New', Courier, monospace;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.border-b {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
}

.bg-zinc-800 {
  background-color: #27272a !important;
}

.bg-zinc-900 {
  background-color: #121214 !important;
}

.border-zinc-700 {
  border-color: #3f3f46 !important;
}

.border-zinc-800 {
  border-color: #27272a !important;
}

.text-zinc-500 {
  color: #71717a !important;
}

:deep(.v-window) {
  min-height: 200px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
</style>