<template>
  <div>
    <div class="d-flex justify-space-between align-center mb-4">
      <div class="text-body-2 text-grey font-weight-medium">Zugeordnete Aktionen</div>

      <v-menu location="bottom end">
        <template v-slot:activator="{ props }">
          <v-btn
              v-bind="props"
              size="small"
              color="primary"
              variant="tonal"
              prepend-icon="mdi-plus"
              class="text-none rounded-lg font-weight-medium px-3"
          >
            Aktion hinzufügen
          </v-btn>
        </template>
        <v-list bg-color="#18181b" class="border border-zinc-700 rounded-lg mt-1" density="compact">
          <v-list-item
              v-for="a in actionsLibrary"
              :key="a.title"
              :prepend-icon="a.icon"
              :title="a.title"
              class="action-menu-item text-body-2"
              @click="assignAction(a)"
          ></v-list-item>
        </v-list>
      </v-menu>
    </div>

    <div
        v-if="boundActionsList.length === 0"
        class="pa-6 border-dashed rounded-lg border-zinc-700 text-center text-grey mb-6 bg-zinc-800 bg-opacity-30"
    >
      <div class="text-body-2">Noch keine Aktionen zugewiesen</div>
    </div>

    <div class="d-flex flex-column gap-3 mb-8">
      <v-card
          v-for="item in boundActionsList"
          :key="item.triggerValue"
          color="#18181b"
          variant="flat"
          class="border border-zinc-800 rounded-lg overflow-hidden"
      >
        <div class="d-flex align-center justify-space-between px-3 py-2 bg-zinc-800 border-b border-zinc-700">
          <div class="d-flex align-center flex-grow-1 overflow-hidden mr-3">
            <v-icon :icon="item.icon" color="primary" size="small" class="mr-2 flex-shrink-0"></v-icon>
            <span class="text-body-2 font-weight-bold text-white text-truncate text-help">
              {{ item.actionName }}
              <v-tooltip activator="parent" location="top" open-delay="250">
                {{ item.actionName }}
              </v-tooltip>
            </span>
          </div>

          <div class="d-flex align-center flex-shrink-0">
            <v-select
                :model-value="item.triggerValue"
                :items="triggerOptions"
                variant="plain"
                density="compact"
                hide-details
                class="compact-trigger-select mr-1"
                @update:model-value="(newVal: TriggerType) => moveActionInList(item.triggerValue, newVal)"
            ></v-select>

            <v-btn
                size="small"
                color="grey-darken-1"
                variant="text"
                icon="mdi-close"
                class="hover-error"
                style="width: 28px; height: 28px;"
                @click="unbindSpecificAction(item.triggerValue)"
            ></v-btn>
          </div>
        </div>

        <div class="px-4 py-3" v-if="item.hasSettings">

          <div v-if="item.hasKey" class="d-flex align-center justify-space-between">
            <div class="text-body-2 text-grey">Tastenkombination</div>
            <v-select
                :model-value="item.key"
                :items="fKeys"
                variant="underlined"
                density="compact"
                hide-details
                class="compact-key-select"
                @update:model-value="(val) => updateActionKey(item.triggerValue, val)"
            ></v-select>
          </div>

          <div v-if="item.hasStep">
            <div class="d-flex justify-space-between align-center mb-1">
              <div class="text-body-2 text-grey">Intervall</div>
              <span
                  v-if="editingStepTrigger !== item.triggerValue"
                  class="text-body-2 text-white font-weight-bold edit-trigger"
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
                track-color="zinc-700"
                @update:model-value="(val) => updateActionStep(item.triggerValue, val)"
            ></v-slider>
          </div>

          <div v-if="item.isToggleApp" class="mt-1">
            <div class="d-flex justify-space-between align-center mb-1">
              <div class="text-body-2 text-grey">Prozess auswählen</div>
              <v-btn
                  icon="mdi-refresh"
                  variant="text"
                  size="x-small"
                  color="grey"
                  title="Liste aktualisieren"
                  @click="fetchProcesses"
              ></v-btn>
            </div>
            <v-autocomplete
                :model-value="item.process_name"
                :items="activeProcesses"
                variant="underlined"
                density="compact"
                hide-details
                placeholder="Suche nach .exe..."
                class="text-white"
                @update:model-value="(val) => updateActionProcess(item.triggerValue, val)"
            ></v-autocomplete>
          </div>
        </div>

        <div class="px-3 py-2 bg-zinc-800 bg-opacity-30 d-flex align-center justify-center" v-else>
          <v-icon icon="mdi-information-outline" size="x-small" color="grey" class="mr-2"></v-icon>
          <div class="text-caption text-grey font-italic" style="opacity: 0.6;">
            Keine weiteren Einstellungen erforderlich
          </div>
        </div>

      </v-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import {
  updateActionMapping,
  removeActionMapping,
  getActiveProcesses,
  type TriggerType
} from '@/services/streamdeckCommands';

const store = useStreamDeckStore();

// --- STATE ---
const editingStepTrigger = ref<TriggerType | null>(null);
const activeProcesses = ref<string[]>([]);
const fKeys = Array.from({ length: 12 }, (_, i) => `F${i + 13}`);

const actionsLibrary = [
  { title: 'Taste drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F13' } },
  { title: 'Spotify Volume', icon: 'mdi-spotify', config: { type: 'SpotifyVolume', step: 5 } },
  { title: 'Master Volume', icon: 'mdi-volume-high', config: { type: 'MasterVolume', step: 5 } },
  { title: 'Audio Toggle', icon: 'mdi-swap-horizontal', config: { type: 'ToggleAudio', device1: 'HyperX', device2: 'Speakers' } },
  { title: 'App Audio (Toggle)', icon: 'mdi-volume-off', config: { type: 'ToggleAppAudio', process_name: '' } },
  { title: 'Global Mute (Toggle)', icon: 'mdi-volume-mute', config: { type: 'ToggleMasterMute' } }
];

// --- COMPUTED ---
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

  return Object.entries(actionsMap).map(([triggerValue, setup]) => {
    const hasSettings = setup?.config && ('step' in setup.config || 'key' in setup.config || setup.config.type === 'ToggleAppAudio');
    return {
      triggerValue: triggerValue as TriggerType,
      actionName: setup?.action || 'Unbekannt',
      icon: setup?.icon || 'mdi-help',
      hasStep: setup?.config && 'step' in setup.config,
      step: setup?.config?.step,
      hasKey: setup?.config && 'key' in setup.config,
      key: setup?.config?.key,
      isToggleApp: setup?.config && setup.config.type === 'ToggleAppAudio',
      process_name: setup?.config?.process_name,
      hasSettings
    };
  });
});

// --- METHODS ---
const fetchProcesses = async () => {
  try {
    activeProcesses.value = await getActiveProcesses();
  } catch (error) {
    console.error("Prozesse konnten nicht geladen werden:", error);
  }
};

const startEditingStep = (trigger: TriggerType) => { editingStepTrigger.value = trigger; };
const stopEditingStep = () => { editingStepTrigger.value = null; };

const assignAction = async (action: any) => {
  if (!store.selectedElementId) return;
  const usedTriggers = Object.keys(store.activeProfile?.keys[store.selectedElementId]?.actions || {});
  let targetTrigger: TriggerType = triggerOptions.value[0].value as TriggerType;

  for (const opt of triggerOptions.value) {
    if (!usedTriggers.includes(opt.value)) {
      targetTrigger = opt.value as TriggerType;
      break;
    }
  }

  const config = { ...action.config };
  if ('step' in config) config.step = targetTrigger.includes('Left') ? -5 : 5;

  store.updateElementAction(store.selectedElementId, targetTrigger, { action: action.title, icon: action.icon, config: config });
  try { await updateActionMapping(store.selectedElementId, targetTrigger, config); } catch (e) { console.error(e); }
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

const updateActionProcess = async (trigger: TriggerType, name: string) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, process_name: name };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

onMounted(() => {
  fetchProcesses();
});
</script>

<style scoped>
.gap-3 { gap: 12px; }

.text-help { cursor: help; }

.compact-trigger-select { width: 120px; }
.compact-trigger-select :deep(.v-field__input) {
  font-size: 0.8rem !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
  min-height: 28px !important;
  color: #a1a1aa !important;
}
.compact-trigger-select :deep(.v-field__append-inner) { padding-top: 0 !important; align-items: center; }

.compact-key-select { max-width: 80px; }
.compact-key-select :deep(.v-field__input) {
  font-size: 0.875rem !important;
  text-align: right;
  color: #6366f1 !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

.action-menu-item:hover {
  background: rgba(99, 102, 241, 0.1) !important;
  color: #6366f1 !important;
}

.hover-error:hover {
  color: #ef4444 !important;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 50%;
}

.edit-trigger {
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.03);
  transition: all 0.2s;
}
.edit-trigger:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #6366f1 !important;
}

.inline-input-wrapper { width: 55px; margin-top: -6px; }
.inline-input-wrapper :deep(input) {
  text-align: right;
  font-size: 0.875rem !important;
  font-weight: bold;
  color: white !important;
  padding-bottom: 2px !important;
}
.inline-input-wrapper :deep(input[type="number"]::-webkit-outer-spin-button),
.inline-input-wrapper :deep(input[type="number"]::-webkit-inner-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
</style>