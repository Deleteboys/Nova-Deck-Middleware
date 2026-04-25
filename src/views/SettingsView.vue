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

      <v-card color="#18181b" variant="flat" class="border border-zinc-800 rounded-lg pa-6 mb-10">
        <v-window v-model="updatePhase" :touch="false">

          <v-window-item :value="1">
            <div class="text-center mb-6">
              <v-icon size="40" color="primary" class="mb-3">mdi-tray-arrow-down</v-icon>
              <h3 class="text-h6 font-weight-bold text-white mb-1">Update verfügbar</h3>
              <p class="text-body-2 text-grey">Eine neue Firmware-Version steht bereit.</p>
            </div>

            <div class="d-flex align-center justify-center pa-5 mb-6 border-dashed rounded-lg border-zinc-700 bg-zinc-800 bg-opacity-30">
              <div class="text-center flex-1-1-0">
                <div class="text-caption text-uppercase text-grey mb-1">Aktuell</div>
                <div class="text-subtitle-1 font-weight-bold font-monospace text-white">{{ currentVersion }}</div>
              </div>

              <v-icon color="grey-darken-1" size="large" class="mx-2 opacity-50">mdi-arrow-right</v-icon>

              <div class="text-center flex-1-1-0">
                <div class="text-caption text-uppercase text-grey mb-1">Neu</div>
                <div class="text-subtitle-1 font-weight-bold text-primary font-monospace">{{ newVersion }}</div>
              </div>
            </div>

            <v-btn
                color="primary"
                block
                size="large"
                rounded="lg"
                class="font-weight-medium text-none"
                elevation="4"
                @click="startUpdate"
            >
              Jetzt aktualisieren
            </v-btn>
          </v-window-item>

          <v-window-item :value="2">
            <div class="text-center py-4">
              <v-progress-circular
                  indeterminate
                  color="primary"
                  size="48"
                  width="4"
                  class="mb-6"
              ></v-progress-circular>

              <h3 class="text-subtitle-1 font-weight-medium text-white mb-6">{{ statusMessage }}</h3>

              <v-progress-linear
                  v-model="updateProgress"
                  color="primary"
                  height="6"
                  rounded
                  bg-color="zinc-800"
                  class="mb-8"
              ></v-progress-linear>

              <v-alert
                  type="error"
                  variant="tonal"
                  class="text-left text-body-2 font-weight-medium rounded-lg"
                  icon="mdi-power-plug-off"
              >
                Bitte trenne das Deck nicht vom Strom oder PC!
              </v-alert>
            </div>
          </v-window-item>

          <v-window-item :value="3">
            <div class="text-center py-6">
              <v-icon color="success" size="64" class="mb-4">mdi-check-circle-outline</v-icon>
              <h3 class="text-h6 font-weight-bold text-success mb-2">Update erfolgreich!</h3>
              <p class="text-body-2 text-grey mb-8">Dein Gerät wurde aktualisiert und startet neu.</p>

              <v-btn
                  variant="tonal"
                  color="primary"
                  block
                  size="large"
                  rounded="lg"
                  class="text-none"
                  @click="resetUpdate"
              >
                Zurück
              </v-btn>
            </div>
          </v-window-item>

        </v-window>
      </v-card>

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

        <div class="d-flex justify-space-between align-center px-4 py-3">
          <div class="text-body-2 text-grey">Vibration standardmäßig an</div>
          <v-switch color="primary" hide-details density="compact" inset style="flex: 0 0 auto;"></v-switch>
        </div>

      </v-card>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { startBootloader } from "@/services/streamdeckCommands";

// State für das Update-UI
const updatePhase = ref(1); // 1 = Info, 2 = Laden, 3 = Erfolgreich
const updateProgress = ref(0);
const statusMessage = ref("");

// Mock-Daten für die Versionen
const currentVersion = ref("v1.0.4");
const newVersion = ref("v1.1.0");

const startUpdate = async () => {
  // UI auf Ladebildschirm umstellen
  updatePhase.value = 2;
  updateProgress.value = 0;
  statusMessage.value = "Starte Bootloader...";

  try {
    // 1. Bootloader des Geräts triggern
    await startBootloader();
    statusMessage.value = "Flashe Firmware... (0%)";

    // 2. Simulation des Flash-Vorgangs (Fortschrittsbalken)
    const interval = setInterval(() => {
      updateProgress.value += Math.random() * 4;

      if (updateProgress.value > 30 && updateProgress.value < 70) {
        statusMessage.value = `Flashe Firmware... (${Math.round(updateProgress.value)}%)`;
      } else if (updateProgress.value >= 70) {
        statusMessage.value = "Überprüfe Installation...";
      }

      // 3. Update abgeschlossen
      if (updateProgress.value >= 100) {
        updateProgress.value = 100;
        clearInterval(interval);

        setTimeout(() => {
          updatePhase.value = 3;
          currentVersion.value = newVersion.value; // Lokale Version visuell aktualisieren
        }, 800);
      }
    }, 150);

  } catch (error) {
    // Falls der Bootloader-Befehl fehlschlägt
    updatePhase.value = 1;
    alert(`Firmware-Update konnte nicht gestartet werden: ${String(error)}`);
  }
};

const resetUpdate = () => {
  updatePhase.value = 1;
  updateProgress.value = 0;
};
</script>

<style scoped>
/* Typography & Helpers */
.uppercase {
  text-transform: uppercase;
}
.tracking-widest {
  letter-spacing: 0.1em !important;
}

.font-monospace {
  font-family: 'Courier New', Courier, monospace;
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar { width: 6px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 10px; }
.custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(255, 255, 255, 0.2); }

/* Borders & Colors */
.border-b {
  border-bottom: 1px solid rgba(255, 255, 255, 0.08) !important;
}

.border-dashed {
  border-style: dashed !important;
  border-width: 1px;
  border-color: rgba(255, 255, 255, 0.15) !important;
}

.bg-zinc-800 {
  background-color: #27272a !important;
}

.border-zinc-700 {
  border-color: #3f3f46 !important;
}

.border-zinc-800 {
  border-color: #27272a !important;
}

.opacity-50 {
  opacity: 0.5;
}

/* Verhindert, dass das V-Window seine Höhe wild animiert */
:deep(.v-window) {
  min-height: 280px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
</style>