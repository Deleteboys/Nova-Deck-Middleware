<template>
  <div class="pa-0 d-flex flex-column fill-height bg-zinc-900 overflow-hidden">

    <div class="flex-grow-1 overflow-y-auto pa-4">

      <section v-if="store.selectedElementId" class="mb-6">
        <h3 class="text-subtitle-2 mb-4 text-primary uppercase tracking-wider">
          Konfiguration: {{ store.selectedElementId }}
        </h3>

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

        <div class="text-caption text-grey mb-3">Zugewiesene Aktionen:</div>

        <div
            v-if="boundActionsList.length === 0"
            class="pa-8 border-dashed rounded-lg border-zinc-700 text-center text-grey mb-6"
        >
          <v-icon icon="mdi-gesture-tap" size="large" class="mb-2 opacity-50"></v-icon>
          <div class="text-body-2">Klicke unten auf eine Aktion, um sie hinzuzufügen.</div>
        </div>

        <div class="d-flex flex-column gap-4 mb-8">
          <v-card
              v-for="item in boundActionsList"
              :key="item.triggerValue"
              color="#18181b"
              variant="flat"
              class="border border-zinc-700 rounded-lg overflow-hidden"
          >
            <div class="d-flex align-center justify-space-between pa-3 bg-zinc-800">
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
                  label="Auslöser"
                  variant="outlined"
                  density="compact"
                  bg-color="black"
                  hide-details
                  class="mb-3"
                  @update:model-value="(newVal: TriggerType) => moveActionInList(item.triggerValue, newVal)"
              ></v-select>

              <div v-if="item.hasKey" class="bg-black pa-3 rounded border border-zinc-800">
                <div class="text-caption text-primary mb-1">Taste wählen</div>
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
                  <div class="text-caption text-primary">Intervall (Step)</div>

                  <span
                      v-if="editingStepTrigger !== item.triggerValue"
                      class="text-caption text-white font-weight-bold edit-trigger"
                      title="Klicken zur direkten Eingabe"
                      @click="startEditingStep(item.triggerValue)"
                  >
                    {{ item.step > 0 ? '+' : '' }}{{ item.step }}%
                  </span>

                  <div v-else class="inline-input-wrapper">
                    <v-text-field
                        :model-value="item.step"
                        type="number"
                        density="compact"
                        variant="underlined"
                        hide-details
                        autofocus
                        color="primary"
                        @update:model-value="(val) => updateActionStep(item.triggerValue, Number(val))"
                        @blur="stopEditingStep"
                        @keyup.enter="stopEditingStep"
                    ></v-text-field>
                  </div>
                </div>

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
              </div>
            </div>
          </v-card>
        </div>
      </section>

      <div v-else class="pa-10 text-center border-dashed rounded-lg border-zinc-700 text-grey mb-6">
        <v-icon icon="mdi-mouse-move-vertical" class="mb-2 opacity-50"></v-icon>
        <p>Wähle ein Element auf dem Deck aus</p>
      </div>

      <v-divider class="mb-6"></v-divider>

      <section>
        <div class="text-overline mb-4 text-grey">Aktionen Bibliothek</div>
        <v-list bg-color="transparent" density="compact" nav class="pa-0">
          <v-list-item
              v-for="a in actionsLibrary"
              :key="a.title"
              :prepend-icon="a.icon"
              :title="a.title"
              :disabled="!store.selectedElementId"
              @click="assignAction(a)"
              class="mb-2 rounded-lg action-card-item"
          >
          </v-list-item>
        </v-list>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import { updateActionMapping, removeActionMapping, type TriggerType } from '@/services/streamdeckCommands';

const store = useStreamDeckStore();
const buttonLabel = ref('');

// --- State und Funktionen für das Inline-Editing (Step) ---
const editingStepTrigger = ref<TriggerType | null>(null);

const startEditingStep = (trigger: TriggerType) => {
  editingStepTrigger.value = trigger;
};

const stopEditingStep = () => {
  editingStepTrigger.value = null;
};
// ----------------------------------------------------------

const fKeys = Array.from({ length: 12 }, (_, i) => `F${i + 13}`);

const actionsLibrary = [
  { title: 'Taste drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F13' } },
  { title: 'Spotify Volume', icon: 'mdi-spotify', config: { type: 'SpotifyVolume', step: 5 } },
  { title: 'Master Volume', icon: 'mdi-volume-high', config: { type: 'MasterVolume', step: 5 } },
  { title: 'Audio Toggle', icon: 'mdi-swap-horizontal', config: { type: 'ToggleAudio', device1: 'HyperX', device2: 'Speakers' } }
];

const triggerOptions = computed(() => {
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

const assignAction = async (action: any) => {
  if (!store.selectedElementId) return;

  const usedTriggers = Object.keys(store.activeProfile?.keys[store.selectedElementId]?.actions || {});
  let targetTrigger: TriggerType = triggerOptions.value[0].value;

  for (const opt of triggerOptions.value) {
    if (!usedTriggers.includes(opt.value)) {
      targetTrigger = opt.value;
      break;
    }
  }

  const config = { ...action.config };
  if ('step' in config) config.step = targetTrigger.includes('Left') ? -5 : 5;

  store.updateElementAction(store.selectedElementId, targetTrigger, {
    action: action.title,
    icon: action.icon,
    config: config
  });

  try {
    await updateActionMapping(store.selectedElementId, targetTrigger, config);
  } catch (e) { console.error(e); }
};

const moveActionInList = async (oldTrigger: TriggerType, newTrigger: TriggerType) => {
  if (oldTrigger === newTrigger || !store.selectedElementId) return;
  const actionData = store.activeProfile?.keys[store.selectedElementId]?.actions?.[oldTrigger];
  if (!actionData) return;

  store.clearElementAction(store.selectedElementId, oldTrigger);
  store.updateElementAction(store.selectedElementId, newTrigger, actionData);

  try {
    await removeActionMapping(store.selectedElementId, oldTrigger);
    await updateActionMapping(store.selectedElementId, newTrigger, actionData.config);
  } catch (e) { console.error(e); }
};

const updateActionStep = async (trigger: TriggerType, newStep: number) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, step: newStep };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

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
.gap-4 { gap: 16px; }

/* Custom Scrollbar Styling (optional) */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

.action-card-item {
  background: rgba(255, 255, 255, 0.03) !important;
  border: 1px solid rgba(255, 255, 255, 0.08);
  transition: all 0.2s ease-in-out;
}

.action-card-item:hover:not(.v-list-item--disabled) {
  border-color: var(--v-primary-base, #3b82f6);
  background: rgba(59, 130, 246, 0.08) !important;
  transform: translateY(-1px);
}

/* --- Styles für Inline Editing in den Cards --- */
.edit-trigger {
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.edit-trigger:hover {
  background: rgba(255, 255, 255, 0.1);
}

.inline-input-wrapper {
  width: 50px;
  margin-top: -8px;
}

.inline-input-wrapper :deep(input[type="number"]::-webkit-outer-spin-button),
.inline-input-wrapper :deep(input[type="number"]::-webkit-inner-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
.inline-input-wrapper :deep(input[type="number"]) {
  -moz-appearance: textfield;
  text-align: right;
  color: white !important;
}
</style>