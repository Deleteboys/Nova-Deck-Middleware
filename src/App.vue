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
import { onMounted } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import TopBar from "./components/layout/TopBar.vue";

const store = useStreamDeckStore();

onMounted(() => {
  // Startet das Tracking von LED-Änderungen im Hintergrund
  store.initHardwareWatcher();

  console.log("Hardware-Synchronisation aktiv.");
});
</script>

<style>
/* Globale Scrollbar-Verschönerung passend zum Board-Design */
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: #111113; }
::-webkit-scrollbar-thumb { background: #27272a; border-radius: 10px; }
::-webkit-scrollbar-thumb:hover { background: #3f3f46; }
</style>