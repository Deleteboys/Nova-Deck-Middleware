<template>
  <v-app theme="dark">
    <v-navigation-drawer permanent rail expand-on-hover color="surface-variant">
      <v-list density="compact" nav>
        <v-list-item prepend-icon="mdi-view-dashboard" title="Dashboard" value="dash"></v-list-item>
        <v-list-item prepend-icon="mdi-tune-vertical" title="Audio Mixer" value="audio"></v-list-item>
        <v-list-item prepend-icon="mdi-cog" title="Settings" value="settings"></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="bg-grey-darken-4">
      <v-container fluid class="pa-8">

        <div class="text-h5 font-weight-bold mb-4 text-primary">Buttons (Befehle)</div>
        <v-row class="mb-8">
          <v-col v-for="n in 8" :key="'btn-' + n" cols="3">
            <v-hover v-slot="{ isHovering, props }">
              <v-card
                  v-bind="props"
                  :elevation="isHovering ? 12 : 2"
                  :class="['button-card', isHovering ? 'bg-primary' : 'bg-surface']"
                  @click="openButtonConfig(n)"
              >
                <div class="d-flex flex-column align-center justify-center fill-height py-4">
                  <v-icon size="x-large" class="mb-2">{{ buttonConfigs[n-1].icon }}</v-icon>
                  <span class="text-caption text-uppercase font-weight-black">
                    {{ buttonConfigs[n-1].label }}
                  </span>
                </div>
              </v-card>
            </v-hover>
          </v-col>
        </v-row>

        <v-divider class="mb-8"></v-divider>

        <div class="text-h5 font-weight-bold mb-4 text-secondary">Drehregler (Audio Control)</div>
        <v-row>
          <v-col v-for="n in 4" :key="'knob-' + n" cols="3">
            <v-card class="bg-surface pa-4" border>
              <div class="d-flex align-center mb-4">
                <v-avatar color="secondary" size="32" class="mr-3">
                  <v-icon size="small">mdi-knob</v-icon>
                </v-avatar>
                <div class="text-subtitle-2 font-weight-bold">Regler {{ n }}</div>
              </div>

              <v-select
                  v-model="knobConfigs[n-1].targetApp"
                  :items="availableApps"
                  label="Programm wählen"
                  density="compact"
                  variant="outlined"
                  hide-details
                  class="mb-4"
              ></v-select>

              <v-progress-linear
                  v-model="knobConfigs[n-1].volume"
                  color="secondary"
                  height="8"
                  rounded
              ></v-progress-linear>
              <div class="text-right text-caption mt-1 text-grey">
                {{ knobConfigs[n-1].volume }}%
              </div>
            </v-card>
          </v-col>
        </v-row>

      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface ButtonConfig {
  id: number
  label: string
  icon: string
  action: string
}

interface KnobConfig {
  id: number
  targetApp: string
  volume: number
}

// Beispiel-Daten für die 8 Buttons
const buttonConfigs = ref<ButtonConfig[]>(
    Array.from({ length: 8 }, (_, i) => ({
      id: i + 1,
      label: `Button ${i + 1}`,
      icon: 'mdi-gesture-tap-button',
      action: 'none'
    }))
)

// Beispiel-Daten für die 4 Drehregler
const knobConfigs = ref<KnobConfig[]>(
    Array.from({ length: 4 }, (_, i) => ({
      id: i + 1,
      targetApp: 'System',
      volume: 50
    }))
)

// Liste verfügbarer Programme (Diese würdest du via Rust/Tauri abfragen)
const availableApps = ['System', 'Spotify', 'Discord', 'Browser', 'OBS', 'Game']

const openButtonConfig = (id: number) => {
  console.log('Konfiguriere Button:', id)
  // Hier könntest du einen Dialog öffnen
}
</script>

<style scoped>
.button-card {
  aspect-ratio: 1;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease-in-out;
}

.bg-surface {
  background-color: #1e1e1e !important;
}

.text-primary {
  color: #2196F3 !important;
}

.text-secondary {
  color: #00E676 !important;
}
</style>