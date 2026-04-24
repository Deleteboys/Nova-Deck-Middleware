<template>
  <v-container class="py-10">
    <v-card max-width="600" class="mx-auto pa-6" border variant="flat">
      <h2 class="mb-6">System-Einstellungen</h2>
      <v-switch label="Auto-Start mit Windows" color="primary" hide-details></v-switch>
      <v-switch label="Vibration standardmaessig an" color="primary" hide-details></v-switch>
      <v-divider class="my-6"></v-divider>
      <v-btn color="error" variant="outlined" :loading="updating" @click="runFirmwareUpdate">
        Firmware Update erzwingen
      </v-btn>
      <p v-if="status" :class="statusError ? 'text-error' : 'text-success'" class="mt-3 mb-0">
        {{ status }}
      </p>
    </v-card>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { startBootloader } from "@/services/streamdeckCommands";

const updating = ref(false);
const status = ref("");
const statusError = ref(false);

const runFirmwareUpdate = async () => {
  updating.value = true;
  status.value = "";
  statusError.value = false;

  try {
    await startBootloader();
    status.value = "Bootloader gestartet. Firmware-Update kann jetzt erfolgen.";
  } catch (error) {
    statusError.value = true;
    status.value = `Firmware-Update konnte nicht gestartet werden: ${String(error)}`;
  } finally {
    updating.value = false;
  }
};
</script>
