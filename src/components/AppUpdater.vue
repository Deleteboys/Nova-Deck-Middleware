<template>
  <v-dialog v-model="dialogVisible" persistent max-width="450">
    <v-card color="#18181b" variant="flat" class="border border-zinc-800 rounded-lg overflow-hidden">

      <div class="px-6 py-4 border-b border-zinc-800 bg-zinc-900">
        <div class="d-flex align-center text-caption text-primary uppercase tracking-widest font-weight-bold">
          <v-icon size="small" class="mr-2">mdi-cloud-download-outline</v-icon>
          System Update
        </div>
      </div>

      <div class="pa-6">
        <v-window v-model="updatePhase" :touch="false">

          <v-window-item :value="1">
            <div class="text-center mb-6">
              <v-icon size="40" color="primary" class="mb-3">mdi-tray-arrow-down</v-icon>
              <h3 class="text-h6 font-weight-bold text-white mb-1">Update verfügbar</h3>
              <p class="text-body-2 text-grey">Eine neue Version der Middleware steht bereit.</p>
            </div>

            <div
                class="d-flex align-center justify-center pa-4 mb-6 border-dashed rounded-lg border-zinc-700 bg-zinc-800 bg-opacity-30">
              <div class="text-center flex-1-1-0">
                <div class="text-caption text-uppercase text-grey mb-1">Aktuell</div>
                <div class="text-subtitle-1 font-weight-bold font-monospace text-white">v{{ currentVersion }}</div>
              </div>

              <v-icon color="grey-darken-1" size="large" class="mx-2 opacity-50">mdi-arrow-right</v-icon>

              <div class="text-center flex-1-1-0">
                <div class="text-caption text-uppercase text-grey mb-1">Neu</div>
                <div class="text-subtitle-1 font-weight-bold text-primary font-monospace">v{{
                    updateInfo?.version
                  }}
                </div>
              </div>
            </div>

            <div v-if="updateInfo?.body"
                 class="mb-6 pa-3 rounded bg-zinc-900 border border-zinc-800 text-body-2 text-grey custom-scrollbar"
                 style="max-height: 100px; overflow-y: auto;">
              {{ updateInfo.body }}
            </div>

            <div class="d-flex gap-3">
              <v-btn
                  variant="tonal"
                  color="grey"
                  class="flex-1-1-0 text-none"
                  rounded="lg"
                  @click="dialogVisible = false"
              >
                Später
              </v-btn>
              <v-btn
                  color="primary"
                  class="flex-1-1-0 font-weight-medium text-none"
                  rounded="lg"
                  elevation="4"
                  @click="startUpdate"
              >
                Installieren
              </v-btn>
            </div>
          </v-window-item>

          <v-window-item :value="2">
            <div class="text-center py-4">
              <v-icon size="48" color="primary" class="mb-4 spin-animation">mdi-loading</v-icon>

              <h3 class="text-subtitle-1 font-weight-medium text-white mb-2">{{ statusMessage }}</h3>
              <p class="text-caption text-grey mb-6">{{ downloadDetails }}</p>

              <v-progress-linear
                  v-model="downloadProgress"
                  color="primary"
                  height="6"
                  rounded
                  bg-color="zinc-800"
                  class="mb-4"
              ></v-progress-linear>

              <v-alert
                  type="warning"
                  variant="tonal"
                  class="text-left text-caption font-weight-medium rounded-lg"
                  icon="mdi-power-plug-off"
              >
                Bitte schließe die App nicht während des Vorgangs.
              </v-alert>
            </div>
          </v-window-item>

        </v-window>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {ref, shallowRef, onMounted, onUnmounted} from 'vue';
import {check, type Update} from '@tauri-apps/plugin-updater';
import {relaunch} from '@tauri-apps/plugin-process';
import {getVersion} from '@tauri-apps/api/app';
import {listen} from '@tauri-apps/api/event';

const dialogVisible = ref(false);
const updatePhase = ref(1);

const currentVersion = ref('0.0.0');

/**
 * WICHTIG: shallowRef verhindert, dass Vue das Update-Objekt in einen Proxy verwandelt.
 * Ohne shallowRef kommt es beim Aufruf interner Methoden zu "Private member" Fehlern.
 */
const updateInfo = shallowRef<Update | null>(null);

const statusMessage = ref('Lade Update herunter...');
const downloadDetails = ref('');
const downloadProgress = ref(0);

let unlistenUpdateTrigger: (() => void) | null = null;

const checkForUpdates = async (isManualCheck = false) => {
  try {
    const update = await check();

    if (update) {
      updateInfo.value = update;
      updatePhase.value = 1; // Zurücksetzen, falls es schon mal offen war
      dialogVisible.value = true;
    } else if (isManualCheck) {
      alert("Die Middleware ist bereits auf dem neuesten Stand!");
    }
  } catch (error) {
    console.error("Fehler beim Suchen nach Updates:", error);
    if (isManualCheck) {
      alert("Fehler bei der Update-Suche: " + String(error));
    }
  }
};

onMounted(async () => {
  try {
    currentVersion.value = await getVersion();

    // 1. Automatischer Check beim Start der App
    await checkForUpdates(false);

    // 2. Listener für den manuellen Klick in den Settings
    unlistenUpdateTrigger = await listen('trigger-update-check', (event) => {
      // Die Payload { manual: true } wurde in den Settings mitgeschickt
      const isManual = (event.payload as any)?.manual || true;
      checkForUpdates(isManual);
    });
  } catch (error) {
    console.error("Fehler bei der Initialisierung des Updaters:", error);
  }
});

onUnmounted(() => {
  if (unlistenUpdateTrigger) unlistenUpdateTrigger();
});

const startUpdate = async () => {
  if (!updateInfo.value) return;

  updatePhase.value = 2;
  let downloaded = 0;
  let contentLength = 0;

  try {
    // Da wir shallowRef nutzen, ist .value das originale Tauri-Objekt
    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0;
          statusMessage.value = 'Lade Dateien herunter...';
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          if (contentLength > 0) {
            downloadProgress.value = (downloaded / contentLength) * 100;
            const dlMb = (downloaded / 1024 / 1024).toFixed(1);
            const totalMb = (contentLength / 1024 / 1024).toFixed(1);
            downloadDetails.value = `${dlMb} MB / ${totalMb} MB`;
          }
          break;
        case 'Finished':
          statusMessage.value = 'Installiere Update...';
          downloadDetails.value = 'Die App wird in Kürze neu gestartet.';
          downloadProgress.value = 100;
          break;
      }
    });

    await relaunch();
  } catch (error) {
    console.error("Fehler bei der Installation:", error);
    statusMessage.value = 'Fehler beim Update!';
    downloadDetails.value = String(error);

    statusMessage.value = 'Update-Fehler';
    downloadDetails.value = String(error);
    updatePhase.value = 2;
    dialogVisible.value = true;
  }
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

.gap-3 {
  gap: 12px;
}

.bg-zinc-800 {
  background-color: #27272a !important;
}

.bg-zinc-900 {
  background-color: #121214 !important;
}

.border-b {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
}

.border-zinc-700 {
  border-color: #3f3f46 !important;
}

.border-zinc-800 {
  border-color: #27272a !important;
}

.border-dashed {
  border-style: dashed !important;
  border-width: 1px;
  border-color: rgba(255, 255, 255, 0.15) !important;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 10px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

.spin-animation {
  animation: spin 1.5s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}

:deep(.v-window) {
  min-height: 250px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
</style>