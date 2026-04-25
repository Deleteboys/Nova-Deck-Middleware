<template>
  <div class="dashboard-layout bg-neutral-950">

    <aside class="sidebar bg-zinc-900 border-right overflow-y-auto custom-scrollbar">
      <ActionSidebar />
    </aside>

    <main class="center-stage bg-zinc-950 d-flex align-center justify-center overflow-y-auto custom-scrollbar">
      <HardwareBoard />
    </main>

    <aside class="right-panel bg-zinc-900 border-left overflow-hidden">
      <ConfigPanel />
    </aside>

  </div>
</template>

<script setup lang="ts">
import ActionSidebar from '@/components/config/ActionSidebar.vue'
import HardwareBoard from '@/components/config/HardwareBoard.vue'
import ConfigPanel from '@/components/config/ConfigPanel.vue'
</script>

<style scoped>
/* DAS IST DER FIX: Ein hartes 100%-Flex-Layout */
.dashboard-layout {
  display: flex;
  flex-direction: row;
  height: 100%;
  width: 100%;
  overflow: hidden; /* Nichts bricht aus */
}

/* Feste Breiten verhindern, dass das Grid verrutscht */
.sidebar {
  width: 20%;
  min-width: 250px;
  flex-shrink: 0;
}

.center-stage {
  flex-grow: 1; /* Nimmt den gesamten restlichen Platz ein */
  background: radial-gradient(circle at center, #18181b 0%, #09090b 100%);
}

.right-panel {
  width: 20%;
  min-width: 320px;
  flex-shrink: 0;
}

/* Farben */
.bg-zinc-950 { background-color: #09090b !important; }
.bg-zinc-900 { background-color: #121214 !important; }
.border-right { border-right: 1px solid rgba(255, 255, 255, 0.05) !important; }
.border-left { border-left: 1px solid rgba(255, 255, 255, 0.05) !important; }
</style>