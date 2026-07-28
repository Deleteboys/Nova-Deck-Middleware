<template>
  <div>
    <div class="d-flex justify-space-between align-center mb-4">
      <div class="text-body-2 text-grey font-weight-medium">Zugeordnete Aktionen</div>

      <v-menu location="bottom end" :close-on-content-click="false">
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

        <v-list
            bg-color="#18181b"
            class="border border-zinc-700 rounded-lg mt-1"
            density="compact"
            width="350"
        >
          <v-list-group
              v-for="cat in categorizedActions"
              :key="cat.name"
              :value="cat.name"
              color="primary"
          >
            <template v-slot:activator="{ props }">
              <v-list-item
                  v-bind="props"
                  :prepend-icon="cat.icon"
                  :title="cat.name"
                  class="text-body-2 font-weight-bold"
              ></v-list-item>
            </template>

            <v-list-item
                v-for="a in cat.items"
                :key="a.title"
                :prepend-icon="a.icon"
                :title="a.title"
                class="action-menu-item text-caption pl-8"
                @click="assignAction(a)"
            ></v-list-item>
          </v-list-group>
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

          <ActionSettingsKey
              v-if="item.hasKey"
              :action-key="item.key"
              :keys-list="fKeys"
              @update:action-key="(val) => updateActionKey(item.triggerValue, val)"
          />

          <ActionSettingsMedia
              v-if="item.hasMediaKey"
              :action-key="item.key"
              :media-keys="mediaKeys"
              @update:action-key="(val) => updateActionKey(item.triggerValue, val)"
          />

          <ActionSettingsVolume
              v-if="item.hasStep"
              :step="item.step || 0"
              @update:step="(val) => updateActionStep(item.triggerValue, val)"
          />

          <div v-if="item.isAppVolume || item.isToggleAppAudio" class="d-flex align-center justify-space-between px-1 pt-1">
            <span class="text-caption text-grey">App-Wechsler koppeln</span>
            <v-switch
                :model-value="item.useSwitcher"
                density="compact"
                hide-details
                color="primary"
                class="flex-shrink-0"
                style="margin-top: 0"
                @update:model-value="(val) => updateUseSwitcher(item.triggerValue, !!val)"
            />
          </div>

          <div v-if="item.useSwitcher" class="px-1 pb-1">
            <span class="text-caption text-grey-darken-1">
              <v-icon size="x-small" color="primary">mdi-swap-horizontal-circle</v-icon>
              Verwendet das aktuell gewählte Programm des App-Wechslers
            </span>
          </div>

          <ActionSettingsProcess
              v-if="item.needsProcess"
              :process-name="item.process_name"
              :processes="activeProcesses"
              @update:process-name="(val) => updateActionProcess(item.triggerValue, val)"
              @refresh="fetchProcesses"
          />

          <ActionSettingsAudio
              v-if="item.isAudioToggle"
              :device1="item.device1"
              :device2="item.device2"
              :audio-devices="audioDevices"
              @update:device1="(val) => updateActionAudioDevices(item.triggerValue, val, item.device2 || '')"
              @update:device2="(val) => updateActionAudioDevices(item.triggerValue, item.device1 || '', val)"
              @refresh="fetchAudioDevices"
          />

          <ActionSettingsMacro
              v-if="item.isCustomMacro"
              :action-key="item.key"
              @update:action-key="(val) => updateActionKey(item.triggerValue, val)"
          />

          <ActionSettingsAppSwitcher
              v-if="item.isAppSwitcherCycle"
              :apps="item.apps"
              :shared-icon="item.sharedIcon"
              :processes="activeProcesses"
              :is-encoder="!!store.selectedElementId?.startsWith('enc-')"
              @update:apps="(apps, sharedIcon) => updateSwitcherApps(item.triggerValue, apps, sharedIcon)"
          />

        </div>
        <ActionSettingsInfo v-else />
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
  getAudioDevices,
  type TriggerType,
  type AudioDeviceInfo,
  type AppSwitcherEntry,
  getActiveAudioProcesses
} from '@/services/streamdeckCommands';

import ActionSettingsKey from './ActionSettings/ActionSettingsKey.vue';
import ActionSettingsMedia from './ActionSettings/ActionSettingsMedia.vue';
import ActionSettingsVolume from './ActionSettings/ActionSettingsVolume.vue';
import ActionSettingsProcess from './ActionSettings/ActionSettingsProcess.vue';
import ActionSettingsAudio from './ActionSettings/ActionSettingsAudio.vue';
import ActionSettingsInfo from './ActionSettings/ActionSettingsInfo.vue';
import ActionSettingsMacro from "@/components/config/ConfigPanel/ActionSettings/ActionSettingsMacro.vue";
import ActionSettingsAppSwitcher from "@/components/config/ConfigPanel/ActionSettings/ActionSettingsAppSwitcher.vue";

const store = useStreamDeckStore();

const activeProcesses = ref<string[]>([]);
const audioDevices = ref<AudioDeviceInfo[]>([]);
const fKeys = Array.from({ length: 12 }, (_, i) => `F${i + 13}`);

const mediaKeys = [
  { title: 'Play / Pause', value: 'MEDIAPLAYPAUSE' },
  { title: 'Nächster Track', value: 'MEDIANEXT' },
  { title: 'Vorheriger Track', value: 'MEDIAPREV' },
  { title: 'Mute', value: 'MEDIAMUTE' }
];

const categorizedActions = [
  {
    name: 'System & Audio',
    icon: 'mdi-monitor-speaker',
    items: [
      { title: 'Master Volume', icon: 'mdi-volume-high', config: { type: 'MasterVolume', step: 5 } },
      { title: 'Global Mute (Toggle)', icon: 'mdi-volume-mute', config: { type: 'ToggleMasterMute' } },
      { title: 'Switch Audio Device', icon: 'mdi-swap-horizontal', config: { type: 'SwitchAudioDevice', device1: '', device2: '' } },
      { title: 'Taste drücken', icon: 'mdi-keyboard', config: { type: 'PressKey', key: 'F13' } },
      { title: 'Eigenes Makro', icon: 'mdi-keyboard-settings', config: { type: 'CustomMacro', key: '' } },
    ]
  },
  {
    name: 'App Steuerung',
    icon: 'mdi-application-cog',
    items: [
      { title: 'App Audio (Volume)', icon: 'mdi-volume-plus', config: { type: 'AppVolume', process_name: '', step: 5 } },
      { title: 'App Audio (Toggle)', icon: 'mdi-volume-off', config: { type: 'ToggleAppAudio', process_name: '' } },
      { title: 'App Media (Play/Pause)', icon: 'mdi-play-pause', config: { type: 'ToggleAppMedia', process_name: '' } },
    ]
  },
  {
    name: 'Fokus Fenster',
    icon: 'mdi-window-maximize',
    items: [
      { title: 'Current Window (Volume)', icon: 'mdi-monitor-speaker', config: { type: 'ForegroundVolume', step: 5 } },
      { title: 'Aktuelles Fenster (Toggle)', icon: 'mdi-speaker-off', config: { type: 'ToggleForegroundAudio' } },
    ]
  },
  {
    name: 'App-Wechsler',
    icon: 'mdi-swap-horizontal-circle',
    items: [
      { title: 'App-Wechsler', icon: 'mdi-swap-horizontal-circle', config: { type: 'AppSwitcherCycle', apps: [], shared_icon: null, direction: 1 } },
    ]
  },
  {
    name: 'Spotify Integration',
    icon: 'mdi-spotify',
    items: [
      // { title: 'Spotify Volume', icon: 'mdi-volume-high', config: { type: 'SpotifyVolume', step: 5 } },
      { title: 'Song Liken', icon: 'mdi-heart', config: { type: 'SpotifyLikeAction' } },
    ]
  },
  {
    name: 'Media Steuerung',
    icon: 'mdi-play-network',
    items: [
      { title: 'Media Control', icon: 'mdi-play-pause', config: { type: 'MediaControl', key: 'MEDIAPLAYPAUSE' } },
    ]
  }
];

const ENCODER_ORDER: TriggerType[] = ['TurnRight', 'TurnLeft', 'PushTurnRight', 'PushTurnLeft', 'PushPress', 'LongPress'];
const BUTTON_ORDER: TriggerType[] = ['ShortPress', 'DoublePress', 'LongPress'];

const triggerOptions = computed(() => {
  if (store.selectedElementId?.startsWith('enc-')) {
    return [
      { title: 'Rechts drehen', value: 'TurnRight' },
      { title: 'Links drehen', value: 'TurnLeft' },
      { title: 'Drücken + Rechts', value: 'PushTurnRight' },
      { title: 'Drücken + Links', value: 'PushTurnLeft' },
      { title: 'Nur Drücken', value: 'PushPress' },
      { title: 'Lang drücken', value: 'LongPress' }
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

  const list = Object.entries(actionsMap).map(([triggerValue, setup]) => {
    const config = setup?.config;
    const type = config?.type;

    const isAppVolume = type === 'AppVolume';
    const isToggleAppAudio = type === 'ToggleAppAudio';
    const useSwitcher = (isAppVolume || isToggleAppAudio) && !!(config as any)?.use_switcher;
    const hasStep = config && 'step' in config && type !== 'AppSwitcherCycle';
    const hasKey = config && 'key' in config && type === 'PressKey';
    const hasMediaKey = config && 'key' in config && type === 'MediaControl';
    const isCustomMacro = config && 'key' in config && type === 'CustomMacro';
    const needsProcess = (isAppVolume && !useSwitcher) || (isToggleAppAudio && !useSwitcher) || type === 'ToggleAppMedia';
    const isAudioToggle = type === 'SwitchAudioDevice';
    const isAppSwitcherCycle = type === 'AppSwitcherCycle';

    const hasSettings = hasStep || hasKey || hasMediaKey || needsProcess || isAudioToggle || isCustomMacro || isAppSwitcherCycle || isToggleAppAudio;

    return {
      triggerValue: triggerValue as TriggerType,
      actionName: setup?.action || 'Unbekannt',
      icon: setup?.icon || 'mdi-help',
      hasStep,
      step: config?.step,
      hasKey,
      hasMediaKey,
      isCustomMacro,
      key: config?.key,
      needsProcess,
      process_name: config?.process_name,
      isAudioToggle,
      device1: config?.device1,
      device2: config?.device2,
      isAppSwitcherCycle,
      apps: isAppSwitcherCycle ? (config as any).apps as AppSwitcherEntry[] : [],
      sharedIcon: isAppSwitcherCycle ? (config as any).shared_icon as string | null : null,
      isAppVolume,
      isToggleAppAudio,
      useSwitcher,
      hasSettings
    };
  });

  const isEncoder = store.selectedElementId.startsWith('enc-');
  const order = isEncoder ? ENCODER_ORDER : BUTTON_ORDER;

  // AppSwitcherCycle auf TurnRight+TurnLeft: TurnLeft ausblenden, nur TurnRight zeigen
  const bothCycle =
      actionsMap['TurnRight']?.config?.type === 'AppSwitcherCycle' &&
      actionsMap['TurnLeft']?.config?.type === 'AppSwitcherCycle';

  return list
      .filter(item => !(bothCycle && item.triggerValue === 'TurnLeft'))
      .sort((a, b) => order.indexOf(a.triggerValue) - order.indexOf(b.triggerValue));
});

const fetchProcesses = async () => {
  try { activeProcesses.value = await getActiveAudioProcesses(); } catch (e) { console.error(e); }
};

const fetchAudioDevices = async () => {
  try { audioDevices.value = await getAudioDevices(); } catch (e) { console.error(e); }
};

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
  if ('step' in config) {
    config.step = (targetTrigger === 'TurnLeft' || targetTrigger === 'PushTurnLeft') ? -5 : 5;
  }
  if (config.type === 'AppSwitcherCycle') {
    config.direction = (targetTrigger === 'TurnLeft' || targetTrigger === 'PushTurnLeft') ? -1 : 1;
  }

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

const updateActionProcess = async (trigger: TriggerType, name: string) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, process_name: name };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

const updateActionAudioDevices = async (trigger: TriggerType, device1: string, device2: string) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, device1, device2 };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

const updateUseSwitcher = async (trigger: TriggerType, useSwitcher: boolean) => {
  if (!store.selectedElementId) return;
  const currentAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[trigger];
  if (currentAction) {
    const updatedConfig = { ...currentAction.config, use_switcher: useSwitcher };
    store.updateElementAction(store.selectedElementId, trigger, { ...currentAction, config: updatedConfig });
    try { await updateActionMapping(store.selectedElementId, trigger, updatedConfig); } catch (e) { console.error(e); }
  }
};

const updateSwitcherApps = async (trigger: TriggerType, apps: AppSwitcherEntry[], sharedIcon: string | null) => {
  if (!store.selectedElementId) return;
  const elementActions = store.activeProfile?.keys[store.selectedElementId]?.actions || {};
  for (const t of Object.keys(elementActions) as TriggerType[]) {
    const currentAction = elementActions[t];
    if (currentAction?.config?.type === 'AppSwitcherCycle') {
      const updatedConfig = { ...currentAction.config, apps, shared_icon: sharedIcon };
      store.updateElementAction(store.selectedElementId, t, { ...currentAction, config: updatedConfig });
      try { await updateActionMapping(store.selectedElementId, t, updatedConfig); } catch (e) { console.error(e); }
    }
  }
};

const unbindSpecificAction = async (triggerToDelete: TriggerType) => {
  if (!store.selectedElementId) return;
  store.clearElementAction(store.selectedElementId, triggerToDelete);
  try { await removeActionMapping(store.selectedElementId, triggerToDelete); } catch (e) { console.error(e); }

  // AppSwitcherCycle: Gegenseite ebenfalls entfernen
  const paired: Partial<Record<TriggerType, TriggerType>> = { TurnRight: 'TurnLeft', TurnLeft: 'TurnRight' };
  const pairedTrigger = paired[triggerToDelete];
  if (pairedTrigger) {
    const pairedAction = store.activeProfile?.keys[store.selectedElementId]?.actions?.[pairedTrigger];
    if (pairedAction?.config?.type === 'AppSwitcherCycle') {
      store.clearElementAction(store.selectedElementId, pairedTrigger);
      try { await removeActionMapping(store.selectedElementId, pairedTrigger); } catch (e) { console.error(e); }
    }
  }
};

onMounted(() => {
  fetchProcesses();
  fetchAudioDevices();
});
</script>

<style scoped>
.gap-3 { gap: 12px; }
.text-help { cursor: help; }
.compact-trigger-select { width: 120px; }
.compact-trigger-select :deep(.v-field__input) {
  font-size: 0.8rem !important;
  min-height: 28px !important;
  color: #a1a1aa !important;
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
</style>