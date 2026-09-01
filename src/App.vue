<template>
  <v-app theme="dark" class="h-screen overflow-hidden">
    <TopBar />

    <v-main class="d-flex flex-column h-100">
      <router-view v-slot="{ Component }">
        <v-fade-transition mode="out-in">
          <component :is="Component" />
        </v-fade-transition>
      </router-view>
    </v-main>

    <AppUpdater />

  </v-app>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useStreamDeckStore } from '@/stores/streamdeck';
import {
  getConnectionStatus,
  requestDeviceConfig,
  type DeviceConfig
} from '@/services/streamdeckCommands';
import TopBar from "./components/layout/TopBar.vue";
import AppUpdater from "@/components/AppUpdater.vue";
import {attachConsole} from "@tauri-apps/plugin-log";

const store = useStreamDeckStore();
const unlistenCallbacks: UnlistenFn[] = [];

let detachLogs: (() => void) | null = null;

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

onMounted(async () => {

  detachLogs = await attachConsole();
  await store.initLogger();
  // 2. Browser-Konsole abfangen und in den Store leiten
  const consoleMethods = [
    { method: 'error', level: 1 },
    { method: 'warn', level: 2 },
    { method: 'info', level: 3 },
    { method: 'debug', level: 4 },
    { method: 'trace', level: 5 },
    { method: 'log', level: 3 }
  ] as const;

  consoleMethods.forEach(({ method, level }) => {
    const originalFn = (console as any)[method];

    (console as any)[method] = (...args: any[]) => {
      originalFn(...args);

      const message = args
          .map(a => (typeof a === 'object' ? JSON.stringify(a) : String(a)))
          .join(' ');

      store.addLog(message, level);
    };
  });
  store.initHardwareWatcher();

  store.syncActiveProfileMappingsToBackend().catch((error) => {
    console.warn('Persisted mappings could not be synced:', error);
  });

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

watch(
  () => store.currentProfileId,
  () => {
    store.syncActiveProfileMappingsToBackend().catch((error) => {
      console.warn('Profile mappings could not be synced:', error);
    });
  }
);

onUnmounted(() => {
  if (detachLogs) detachLogs();
  for (const unlisten of unlistenCallbacks) {
    unlisten();
  }
});
</script>

<style>
/* Verhindert den globalen Scrollbalken komplett (Bulletproof) */
html { overflow-y: hidden !important; }

/* Globale Scrollbar-Verschonerung */
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: #111113; }
::-webkit-scrollbar-thumb { background: #27272a; border-radius: 10px; }
::-webkit-scrollbar-thumb:hover { background: #3f3f46; }
</style>
