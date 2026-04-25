<template>
  <div class="pa-4 d-flex flex-column fill-height bg-zinc-900">

    <div v-if="store.selectedElementId" class="mb-6">
      <h3 class="text-subtitle-2 mb-4 text-primary">KONFIGURATION: {{ store.selectedElementId }}</h3>

      <v-text-field
          v-model="buttonLabel"
          label="Button Beschriftung"
          variant="outlined"
          density="comfortable"
          class="mb-4"
          @input="saveChanges"
      ></v-text-field>

      <v-select
          v-model="selectedTriggerType"
          :items="triggerOptions"
          label="Auslöser (Event)"
          variant="outlined"
          density="comfortable"
          class="mb-4"
      ></v-select>

      <div v-if="hasStepConfig" class="mb-4">
        <div class="text-caption text-grey mb-1">Lautstärke-Intervall (Step): {{ localStep > 0 ? '+' : '' }}{{ localStep }}%</div>
        <v-slider
            v-model="localStep"
            :min="-50"
            :max="50"
            :step="1"
            hide-details
            thumb-label
            color="primary"
            @end="updateActionStep"
        ></v-slider>
        <v-text-field
            v-model.number="localStep"
            type="number"
            variant="underlined"
            density="compact"
            suffix="%"
            @change="updateActionStep"
        ></v-text-field>
      </div>

      <div class="pa-4 bg-black rounded-lg border border-zinc-700 mb-6">
        <div class="text-caption text-grey mb-3">Zugewiesene Aktionen:</div>

        <div v-if="boundActionsList.length === 0" class="text-body-2 text-grey-darken-1 text-center py-2">
          Noch keine Aktionen zugewiesen.
        </div>

        <div
            v-for="item in boundActionsList"
            :key="item.triggerValue"
            class="d-flex justify-space-between align-center mb-2 pa-2 bg-zinc-900 rounded border border-zinc-800"
        >
          <div class="d-flex align-center">
            <v-icon :icon="item.icon" color="primary" class="mr-3" size="small"></v-icon>
            <div>
              <div class="text-caption text-grey" style="line-height: 1.2;">{{ item.triggerName }}</div>
              <div class="text-body-2">
                {{ item.actionName }}
                <span v-if="item.step !== undefined" class="text-primary">({{ item.step > 0 ? '+' : '' }}{{ item.step }}%)</span>
              </div>
            </div>
          </div>
          <v-btn
              size="x-small"
              color="error"
              variant="text"
              icon="mdi-delete"
              @click="unbindSpecificAction(item.triggerValue)"
          ></v-btn>
        </div>
      </div>
    </div>

    <div v-else class="pa-10 text-center border-dashed rounded-lg text-grey mb-6">
      Wähle ein Element auf dem Deck aus
    </div>

    <v-divider class="mb-6"></v-divider>

    <div class="flex-grow-1 overflow-y-auto">
      <div class="text-overline mb-4 text-grey">Aktionen Bibliothek</div>
      <v-list bg-color="transparent" density="compact" nav>
        <v-list-item
            v-for="a in actionsLibrary"
            :key="a.title"
            :prepend-icon="a.icon"
            :title="a.title"
            :disabled="!store.selectedElementId"
            @click="assignAction(a)"
            class="mb-2 rounded-lg action-card"
        >
        </v-list-item>
      </v-list>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import { updateActionMapping, removeActionMapping, type TriggerType } from '@/services/streamdeckCommands';

const store = useStreamDeckStore();
const buttonLabel = ref('');
const selectedTriggerType = ref<TriggerType>('ShortPress');
const localStep = ref(5);

const actionsLibrary = [
  { title: 'Taste F14 drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F14' } },
  { title: 'Taste F15 drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F15' } },
  { title: 'Spotify Volume', icon: 'mdi-spotify', config: { type: 'SpotifyVolume', step: 5 } },
  { title: 'Master Volume', icon: 'mdi-volume-high', config: { type: 'MasterVolume', step: 5 } },
  { title: 'Audio Toggle', icon: 'mdi-swap-horizontal', config: { type: 'ToggleAudio', device1: 'HyperX', device2: 'Speakers' } }
];

const triggerOptions = computed<{ title: string; value: TriggerType }[]>(() => {
  if (store.selectedElementId?.startsWith('enc-')) {
    return [
      { title: 'Nach Rechts drehen', value: 'TurnRight' },
      { title: 'Nach Links drehen', value: 'TurnLeft' },
      { title: 'Gedrückt + Rechts drehen', value: 'PushTurnRight' },
      { title: 'Gedrückt + Links drehen', value: 'PushTurnLeft' },
      { title: 'Nur Drücken', value: 'PushPress' }
    ];
  }
  return [
    { title: 'Single Click', value: 'ShortPress' },
    { title: 'Double Click', value: 'DoublePress' },
    { title: 'Long Press', value: 'LongPress' }
  ];
});

const triggerDisplayNames: Record<string, string> = {
  'ShortPress': 'Single Click', 'DoublePress': 'Double Click', 'LongPress': 'Long Press',
  'TurnRight': 'Rechts drehen', 'TurnLeft': 'Links drehen',
  'PushTurnRight': 'Drücken + Rechts', 'PushTurnLeft': 'Drücken + Links', 'PushPress': 'Nur Drücken'
};

const hasStepConfig = computed(() => {
  const currentAction = store.activeProfile?.keys[store.selectedElementId!]?.actions?.[selectedTriggerType.value];
  return currentAction?.config && 'step' in currentAction.config;
});

const boundActionsList = computed(() => {
  if (!store.selectedElementId) return [];
  const actionsMap = store.activeProfile?.keys[store.selectedElementId]?.actions;
  if (!actionsMap) return [];

  return Object.entries(actionsMap).map(([triggerValue, setup]) => ({
    triggerValue: triggerValue as TriggerType,
    triggerName: triggerDisplayNames[triggerValue] || triggerValue,
    actionName: setup?.action || 'Unbekannt',
    icon: setup?.icon || 'mdi-help',
    step: setup?.config?.step
  }));
});

// --- HIER IST DIE FUNKTION JETZT OBEN ---
const syncLocalStep = () => {
  const currentAction = store.activeProfile?.keys[store.selectedElementId!]?.actions?.[selectedTriggerType.value];
  if (currentAction?.config && 'step' in currentAction.config) {
    // FEHLERBEHEBUNG: .value statt .ref
    localStep.value = currentAction.config.step;
  } else {
    // Standardwerte basierend auf Drehrichtung vorschlagen
    localStep.value = selectedTriggerType.value.includes('Left') ? -5 : 5;
  }
};

// --- DANN DIE WATCHER ---
watch(() => store.selectedElementId, (newId) => {
  if (newId) {
    buttonLabel.value = store.activeProfile?.keys[newId]?.label || '';
    selectedTriggerType.value = newId.startsWith('enc-') ? 'TurnRight' : 'ShortPress';
    syncLocalStep();
  }
}, { immediate: true });

watch(selectedTriggerType, () => {
  syncLocalStep();
});

// --- RESTLICHE FUNKTIONEN ---
const updateActionStep = async () => {
  const currentAction = store.activeProfile?.keys[store.selectedElementId!]?.actions?.[selectedTriggerType.value];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, step: localStep.value };

    store.updateElementAction(store.selectedElementId!, selectedTriggerType.value, {
      ...currentAction,
      config: updatedConfig
    });

    try {
      await updateActionMapping(store.selectedElementId!, selectedTriggerType.value, updatedConfig);
    } catch (e) { console.error(e); }
  }
};

const saveChanges = () => {
  if (store.selectedElementId) {
    store.updateElementLabel(store.selectedElementId, buttonLabel.value);
  }
};

const assignAction = async (action: any) => {
  if (store.selectedElementId) {
    const config = { ...action.config };
    if ('step' in config) {
      config.step = localStep.value;
    }

    store.updateElementAction(store.selectedElementId, selectedTriggerType.value, {
      action: action.title,
      icon: action.icon,
      config: config
    });

    try {
      await updateActionMapping(store.selectedElementId, selectedTriggerType.value, config);
    } catch (e) { console.error(e); }
  }
};

const unbindSpecificAction = async (triggerToDelete: TriggerType) => {
  if (store.selectedElementId) {
    store.clearElementAction(store.selectedElementId, triggerToDelete);
    try {
      await removeActionMapping(store.selectedElementId, triggerToDelete);
    } catch (e) { console.error(e); }
  }
};
</script>

<style scoped>
.action-card {
  background: rgba(255,255,255,0.03) !important;
  border: 1px solid rgba(255,255,255,0.1);
  transition: border-color 0.2s;
}
.action-card:hover:not(.v-list-item--disabled) {
  border-color: #3b82f6;
}
</style>