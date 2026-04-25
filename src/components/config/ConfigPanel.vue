<template>
  <div class="pa-4 d-flex flex-column fill-height bg-zinc-900">

    <div v-if="store.selectedElementId" class="mb-6">
      <h3 class="text-subtitle-2 mb-4 text-primary">KONFIGURATION: {{ store.selectedElementId }}</h3>

      <v-text-field
          v-model="buttonLabel"
          label="Button Beschriftung"
          variant="outlined"
          density="comfortable"
          class="mb-6"
          hide-details
          @input="saveChanges"
      ></v-text-field>

      <v-divider class="mb-6 border-opacity-25" color="white"></v-divider>

      <div class="mb-2 d-flex justify-space-between align-center">
        <div class="text-caption text-grey">Zugewiesene Aktionen:</div>
      </div>

      <div v-if="boundActionsList.length === 0" class="pa-6 border-dashed rounded-lg border-zinc-700 text-center text-grey mb-6">
        <v-icon icon="mdi-gesture-tap" size="large" class="mb-2 opacity-50"></v-icon>
        <div class="text-body-2">Klicke auf eine Aktion in der Bibliothek, um sie hinzuzufügen.</div>
      </div>

      <div class="d-flex flex-column" style="gap: 16px; margin-bottom: 32px;">
        <v-card
            v-for="item in boundActionsList"
            :key="item.triggerValue"
            color="#18181b"
            elevation="0"
            class="border border-zinc-700 rounded-lg"
        >
          <div class="d-flex align-center justify-space-between pa-3 border-b border-zinc-800 bg-zinc-800">
            <div class="d-flex align-center">
              <v-icon :icon="item.icon" color="primary" class="mr-3" size="small"></v-icon>
              <span class="text-body-2 font-weight-bold text-white">{{ item.actionName }}</span>
            </div>
            <v-btn
                size="x-small"
                color="error"
                variant="text"
                icon="mdi-trash-can-outline"
                @click="unbindSpecificAction(item.triggerValue)"
            ></v-btn>
          </div>

          <div class="pa-3">

            <v-select
                :model-value="item.triggerValue"
                :items="triggerOptions"
                label="Ausgelöst durch:"
                variant="outlined"
                density="compact"
                bg-color="black"
                hide-details
                class="mb-3"
                @update:model-value="(newVal: TriggerType) => moveActionInList(item.triggerValue, newVal)"
            ></v-select>

            <div v-if="item.hasKey" class="bg-black pa-3 rounded border border-zinc-800">
              <div class="text-caption text-primary mb-2">Taste wählen:</div>
              <v-select
                  :model-value="item.key"
                  :items="fKeys"
                  variant="underlined"
                  density="compact"
                  hide-details
                  @update:model-value="(val) => updateActionKey(item.triggerValue, val)"
              ></v-select>
            </div>

            <div v-if="item.hasStep" class="bg-black pa-3 rounded border border-zinc-800">
              <div class="d-flex justify-space-between align-center mb-1">
                <div class="text-caption text-primary">Intervall (Step):</div>
                <div class="text-caption text-white">{{ item.step > 0 ? '+' : '' }}{{ item.step }}%</div>
              </div>

              <div class="d-flex align-center" style="gap: 16px;">
                <v-slider
                    :model-value="item.step"
                    :min="-50"
                    :max="50"
                    :step="1"
                    hide-details
                    color="primary"
                    class="flex-grow-1"
                    @update:model-value="(val) => updateActionStep(item.triggerValue, val)"
                ></v-slider>
                <v-text-field
                    :model-value="item.step"
                    type="number"
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="max-width: 80px;"
                    @update:model-value="(val) => updateActionStep(item.triggerValue, Number(val))"
                ></v-text-field>
              </div>
            </div>

          </div>
        </v-card>
      </div>
    </div>

    <div v-else class="pa-10 text-center border-dashed rounded-lg text-grey mb-6">
      Wähle ein Element auf dem Deck aus
    </div>

    <v-divider class="mb-6"></v-divider>

    <div class="flex-grow-1 overflow-y-auto">
      <div class="text-overline mb-4 text-grey">Aktionen Bibliothek</div>
      <v-list bg-color="transparent" density="compact" nav class="pa-0">
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

const fKeys = Array.from({ length: 12 }, (_, i) => `F${i + 13}`);

const actionsLibrary = [
  { title: 'Taste drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F13' } },
  { title: 'Spotify Volume', icon: 'mdi-spotify', config: { type: 'SpotifyVolume', step: 5 } },
  { title: 'Master Volume', icon: 'mdi-volume-high', config: { type: 'MasterVolume', step: 5 } },
  { title: 'Audio Toggle', icon: 'mdi-swap-horizontal', config: { type: 'ToggleAudio', device1: 'HyperX', device2: 'Speakers' } }
];

// Die verfügbaren Auslöser für das aktuell gewählte Hardware-Element
const triggerOptions = computed<{ title: string; value: TriggerType }[]>(() => {
  if (store.selectedElementId?.startsWith('enc-')) {
    return [
      { title: 'Rechts drehen', value: 'TurnRight' },
      { title: 'Links drehen', value: 'TurnLeft' },
      { title: 'Drücken + Rechts', value: 'PushTurnRight' },
      { title: 'Drücken + Links', value: 'PushTurnLeft' },
      { title: 'Nur Drücken', value: 'PushPress' }
    ];
  }
  return [
    { title: 'Single Click', value: 'ShortPress' },
    { title: 'Double Click', value: 'DoublePress' },
    { title: 'Long Press', value: 'LongPress' }
  ];
});

// Liest alle zugewiesenen Aktionen sauber für das Template aus
const boundActionsList = computed(() => {
  if (!store.selectedElementId) return [];
  const actionsMap = store.activeProfile?.keys[store.selectedElementId]?.actions;
  if (!actionsMap) return [];

  return Object.entries(actionsMap).map(([triggerValue, setup]) => ({
    triggerValue: triggerValue as TriggerType,
    actionName: setup?.action || 'Unbekannt',
    icon: setup?.icon || 'mdi-help',
    hasStep: setup?.config && 'step' in setup.config,
    step: setup?.config?.step,
    hasKey: setup?.config && 'key' in setup.config,
    key: setup?.config?.key
  }));
});

// Lädt nur noch das Label, wenn man ein Element anklickt
watch(() => store.selectedElementId, (newId) => {
  if (newId) {
    buttonLabel.value = store.activeProfile?.keys[newId]?.label || '';
  }
}, { immediate: true });

const saveChanges = () => {
  if (store.selectedElementId) {
    store.updateElementLabel(store.selectedElementId, buttonLabel.value);
  }
};

// Logik zum Hinzufügen einer NEUEN Aktion aus der Bibliothek
const assignAction = async (action: any) => {
  if (!store.selectedElementId) return;

  // 1. Finde einen freien Slot (Auslöser), der noch nicht belegt ist
  const usedTriggers = Object.keys(store.activeProfile?.keys[store.selectedElementId]?.actions || {});
  let targetTrigger: TriggerType = triggerOptions.value[0].value; // Fallback: Erster Auslöser

  for (const opt of triggerOptions.value) {
    if (!usedTriggers.includes(opt.value)) {
      targetTrigger = opt.value;
      break; // Ersten freien Platz gefunden!
    }
  }

  // 2. Setze smarte Standardwerte
  const config = { ...action.config };
  if ('step' in config) config.step = targetTrigger.includes('Left') ? -5 : 5;
  if ('key' in config) config.key = 'F13';

  // 3. Im Store speichern
  store.updateElementAction(store.selectedElementId, targetTrigger, {
    action: action.title,
    icon: action.icon,
    config: config
  });

  // 4. An Rust senden
  try {
    await updateActionMapping(store.selectedElementId, targetTrigger, config);
  } catch (e) { console.error(e); }
};

// Verschiebt eine Aktion auf einen anderen Auslöser
const moveActionInList = async (oldTrigger: TriggerType, newTrigger: TriggerType) => {
  if (oldTrigger === newTrigger || !store.selectedElementId) return;

  const actionData = store.activeProfile?.keys[store.selectedElementId]?.actions?.[oldTrigger];
  if (!actionData) return;

  // Alte löschen
  store.clearElementAction(store.selectedElementId, oldTrigger);
  try { await removeActionMapping(store.selectedElementId, oldTrigger); } catch (e) { console.error(e); }

  // Neue setzen
  store.updateElementAction(store.selectedElementId, newTrigger, actionData);
  try { await updateActionMapping(store.selectedElementId, newTrigger, actionData.config); } catch (e) { console.error(e); }
};

// Aktualisiert den Slider-Wert einer spezifischen Aktion
const updateActionStep = async (trigger: TriggerType, newStep: number) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];

  if (currentAction) {
    const updatedConfig = { ...currentAction.config, step: newStep };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

// Aktualisiert die F-Taste einer spezifischen Aktion
const updateActionKey = async (trigger: TriggerType, newKey: string) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];

  if (currentAction) {
    const updatedConfig = { ...currentAction.config, key: newKey };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

const unbindSpecificAction = async (triggerToDelete: TriggerType) => {
  if (store.selectedElementId) {
    store.clearElementAction(store.selectedElementId, triggerToDelete);
    try { await removeActionMapping(store.selectedElementId, triggerToDelete); } catch (e) { console.error(e); }
  }
};
</script>

<style scoped>
.action-card {
  background: rgba(255,255,255,0.03) !important;
  border: 1px solid rgba(255,255,255,0.1);
  transition: all 0.2s ease;
}
.action-card:hover:not(.v-list-item--disabled) {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.05) !important;
}
</style>