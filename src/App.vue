<template>
  <v-app theme="dark">
    <TopBar />

    <v-main>
      <router-view v-slot="{ Component }">
        <v-fade-transition mode="out-in">
          <component :is="Component" />
        </v-fade-transition>
      </router-view>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useStreamDeckStore } from '@/stores/streamdeck';
import {
  getConnectionStatus,
  requestDeviceConfig,
  type DeviceConfig
} from '@/services/streamdeckCommands';
import TopBar from "./components/layout/TopBar.vue";

const store = useStreamDeckStore();
const unlistenCallbacks: UnlistenFn[] = [];

type PicoConfigEvent = {
  Config: {
    config: DeviceConfig;
  };
};

const extractConfig = (payload: unknown): DeviceConfig | null => {
  if (!payload || typeof payload !== 'object') {
    return null;
  }

  const configEvent = payload as PicoConfigEvent;
  if (!configEvent.Config?.config) {
    return null;
  }

  return configEvent.Config.config;
};

onMounted(() => {
  // Startet das Tracking von LED-Änderungen im Hintergrund
  store.initHardwareWatcher();

  listen<boolean>('pico-connection', async (event) => {
    store.setDeviceConnected(event.payload);
    if (!event.payload) return;

    try {
      await requestDeviceConfig();
    } catch (error) {
      console.warn('Config konnte nach dem Connect nicht angefragt werden:', error);
    }
  }).then((unlisten) => unlistenCallbacks.push(unlisten));

  listen<unknown>('pico-event', (event) => {
    const config = extractConfig(event.payload);
    if (config) {
      store.applyDeviceConfig(config);
    }
  }).then((unlisten) => unlistenCallbacks.push(unlisten));

  getConnectionStatus()
      .then(async (isConnected) => {
        store.setDeviceConnected(isConnected);
        if (isConnected) {
          await requestDeviceConfig();
        }
      })
      .catch(() => {
        store.setDeviceConnected(false);
      });

  console.log("Hardware-Synchronisation aktiv.");
});

onUnmounted(() => {
  for (const unlisten of unlistenCallbacks) {
    unlisten();
  }
});
</script>

<style>
/* Globale Scrollbar-Verschönerung passend zum Board-Design */
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: #111113; }
::-webkit-scrollbar-thumb { background: #27272a; border-radius: 10px; }
::-webkit-scrollbar-thumb:hover { background: #3f3f46; }
</style>
