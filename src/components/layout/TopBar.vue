<template>
  <v-app-bar color="surface" elevation="3" class="px-4">
    <div class="d-flex align-center flex-1-1-0">
      <v-icon color="primary" size="large" class="mr-2">mdi-developer-board</v-icon>
      <div class="text-h6 font-weight-bold text-no-wrap">
        DIY DECK <v-chip size="x-small" color="primary" variant="tonal" class="ml-1">V2</v-chip>
      </div>
    </div>

    <div class="d-flex justify-center gap-2">
      <v-btn
          v-for="nav in navItems"
          :key="nav.id"
          :to="nav.path"
          :color="$route.path === nav.path ? 'primary' : 'default'"
          :variant="$route.path === nav.path ? 'flat' : 'text'"
          class="text-none font-weight-medium"
          rounded="lg"
      >
        {{ nav.label }}
      </v-btn>
    </div>

    <div class="d-flex align-center justify-end flex-1-1-0">
      <v-chip
          size="small"
          :color="store.isDeviceConnected ? 'success' : 'error'"
          variant="tonal"
          class="mr-3"
      >
        <v-icon start size="14">
          {{ store.isDeviceConnected ? 'mdi-lan-connect' : 'mdi-lan-disconnect' }}
        </v-icon>
        {{ store.isDeviceConnected ? 'Deck verbunden' : 'Deck getrennt' }}
      </v-chip>

      <v-slide-x-reverse-transition>
        <v-select
            v-if="$route.name === 'config'"
            v-model="store.currentProfileId"
            :items="store.profiles"
            item-title="name"
            item-value="id"
            label="Profil"
            variant="outlined"
            density="compact"
            hide-details
            style="max-width: 200px;"
            prepend-inner-icon="mdi-account-box-outline"
        ></v-select>
      </v-slide-x-reverse-transition>
    </div>
  </v-app-bar>
</template>

<script setup lang="ts">
import { useStreamDeckStore } from '@/stores/streamdeck'
const store = useStreamDeckStore()

const navItems = [
  { id: 'config', label: 'Konfigurator', path: '/' },
  { id: 'test', label: 'Hardware Test', path: '/test' },
  { id: 'settings', label: 'Settings', path: '/settings' }
]
</script>

<style scoped>
.flex-1-1-0 { flex: 1 1 0; min-width: 0; }
.gap-2 { gap: 8px; }
</style>
