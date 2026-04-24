<template>
  <v-container class="py-10">
    <v-row>
      <v-col cols="12">
        <h2 class="text-h4 mb-2">Hardware Test</h2>
        <p class="text-grey">Teste die LEDs und Funktionen deines Decks.</p>
        <p v-if="status" :class="statusError ? 'text-error' : 'text-success'" class="mt-2">
          {{ status }}
        </p>
      </v-col>
      <v-col v-for="n in 12" :key="n" cols="6" sm="4" md="2">
        <v-btn
            block
            height="80"
            variant="outlined"
            color="primary"
            :loading="loadingLed === n"
            @click="testLed(n)"
        >
          LED {{ n }}
        </v-btn>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { setLed } from "@/services/streamdeckCommands";

const loadingLed = ref<number | null>(null);
const status = ref("");
const statusError = ref(false);

const TEST_COLORS = [
  { r: 255, g: 64, b: 64 },
  { r: 64, g: 160, b: 255 },
  { r: 64, g: 220, b: 120 },
  { r: 255, g: 195, b: 64 },
  { r: 175, g: 120, b: 255 },
  { r: 255, g: 105, b: 180 },
];

const testLed = async (id: number) => {
  loadingLed.value = id;
  status.value = "";
  statusError.value = false;

  const zeroBasedIndex = id - 1;
  const color = TEST_COLORS[zeroBasedIndex % TEST_COLORS.length];

  try {
    await setLed(zeroBasedIndex, color, 220);
    status.value = `LED ${id} wurde gesetzt.`;
  } catch (error) {
    statusError.value = true;
    status.value = `LED ${id} konnte nicht gesetzt werden: ${String(error)}`;
  } finally {
    loadingLed.value = null;
  }
};
</script>
