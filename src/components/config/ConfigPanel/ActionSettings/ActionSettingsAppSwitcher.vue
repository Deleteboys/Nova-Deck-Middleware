<template>
  <div class="pa-2">
    <div class="d-flex align-center justify-space-between mb-3">
      <span class="text-caption text-grey font-weight-medium">Programme</span>
      <v-btn
          size="x-small"
          variant="tonal"
          color="primary"
          prepend-icon="mdi-plus"
          class="text-none"
          @click="addApp"
      >
        App hinzufügen
      </v-btn>
    </div>

    <div v-if="localApps.length === 0" class="text-caption text-grey text-center py-2 mb-3">
      Noch keine Apps – oben hinzufügen
    </div>

    <div v-for="(app, index) in localApps" :key="index" class="d-flex align-center gap-2 mb-2">
      <v-autocomplete
          v-model="app.process_name"
          :items="processes"
          density="compact"
          variant="outlined"
          hide-details
          placeholder="Prozessname"
          class="flex-grow-1"
          style="min-width: 0"
          @update:model-value="emitUpdate"
      />
      <v-select
          v-if="isEncoder"
          :model-value="app.icon || ''"
          :items="iconOptions"
          density="compact"
          variant="outlined"
          hide-details
          style="width: 130px; flex-shrink: 0"
          @update:model-value="(v) => { app.icon = v || null; emitUpdate(); }"
      />
      <v-btn
          size="small"
          variant="text"
          icon="mdi-close"
          color="grey-darken-1"
          class="hover-error flex-shrink-0"
          style="width: 28px; height: 28px;"
          @click="removeApp(index)"
      />
    </div>

    <template v-if="isEncoder">
      <v-divider class="my-3" />

      <div class="d-flex align-center gap-2">
        <span class="text-caption text-grey" style="white-space: nowrap">Standard-Icon:</span>
        <v-select
            :model-value="localSharedIcon || ''"
            :items="sharedIconOptions"
            density="compact"
            variant="outlined"
            hide-details
            class="flex-grow-1"
            @update:model-value="(v) => { localSharedIcon = v || null; emitUpdate(); }"
        />
      </div>
    </template>

    <v-divider class="my-3" />

    <div class="d-flex align-center justify-space-between gap-2">
      <div class="d-flex align-center">
        <span class="text-caption text-grey">Geschlossene Apps überspringen</span>
        <span class="d-inline-flex align-center ml-1 text-help">
          <v-icon icon="mdi-information-outline" size="x-small" color="grey" />
          <v-tooltip activator="parent" location="top" open-delay="250" max-width="260">
            Beim Drehen werden nur Apps mit aktiver Audio-Session angesteuert
          </v-tooltip>
        </span>
      </div>
      <v-switch
          :model-value="localHideClosedApps"
          density="compact"
          hide-details
          color="primary"
          class="flex-shrink-0"
          style="margin-top: 0"
          @update:model-value="(v) => updateHideClosedApps(!!v)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { AppSwitcherEntry } from '@/services/streamdeckCommands';

const props = defineProps<{
  apps: AppSwitcherEntry[];
  sharedIcon: string | null | undefined;
  hideClosedApps: boolean | undefined;
  processes: string[];
  isEncoder: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:apps', apps: AppSwitcherEntry[], sharedIcon: string | null): void;
  (e: 'update:hideClosedApps', hideClosedApps: boolean): void;
}>();

const localApps = ref<AppSwitcherEntry[]>(props.apps.map(a => ({ ...a })));
const localSharedIcon = ref<string | null>(props.sharedIcon ?? null);
const localHideClosedApps = ref<boolean>(props.hideClosedApps ?? false);

watch(() => props.apps, (val) => {
  localApps.value = val.map(a => ({ ...a }));
}, { deep: true });

watch(() => props.sharedIcon, (val) => {
  localSharedIcon.value = val ?? null;
});

watch(() => props.hideClosedApps, (val) => {
  localHideClosedApps.value = val ?? false;
});

const iconOptions = [
  { title: 'Standard (geteilt)', value: '' },
  { title: 'Master', value: 'Master' },
  { title: 'Spotify', value: 'Spotify' },
  { title: 'Discord', value: 'Discord' },
  { title: 'Browser', value: 'Browser' },
  { title: 'Mikrofon', value: 'Mic' },
  { title: 'Kamera', value: 'Camera' },
  { title: 'Play/Pause', value: 'Play_Pause' },
  { title: 'Licht', value: 'Light' },
  { title: 'Aktives Fenster', value: 'Active_Window' },
  { title: 'Jellyfin', value: 'Jellyfin' },
  { title: 'Home', value: 'Home' },
  { title: 'Keins', value: 'None' },
];

const sharedIconOptions = [
  { title: 'Kein Standard-Icon', value: '' },
  { title: 'Master', value: 'Master' },
  { title: 'Spotify', value: 'Spotify' },
  { title: 'Discord', value: 'Discord' },
  { title: 'Browser', value: 'Browser' },
  { title: 'Mikrofon', value: 'Mic' },
  { title: 'Kamera', value: 'Camera' },
  { title: 'Play/Pause', value: 'Play_Pause' },
  { title: 'Licht', value: 'Light' },
  { title: 'Aktives Fenster', value: 'Active_Window' },
  { title: 'Jellyfin', value: 'Jellyfin' },
  { title: 'Home', value: 'Home' },
  { title: 'Keins', value: 'None' },
];

const addApp = () => {
  localApps.value.push({ process_name: '', icon: null });
  emitUpdate();
};

const removeApp = (index: number) => {
  localApps.value.splice(index, 1);
  emitUpdate();
};

const emitUpdate = () => {
  emit('update:apps', localApps.value.map(a => ({ ...a })), localSharedIcon.value);
};

const updateHideClosedApps = (value: boolean) => {
  localHideClosedApps.value = value;
  emit('update:hideClosedApps', value);
};
</script>

<style scoped>
.gap-2 { gap: 8px; }
.text-help { cursor: help; }
.hover-error:hover {
  color: #ef4444 !important;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 50%;
}
</style>
