<template>
  <div class="app-root">
    <header class="top-bar">
      <div class="top-bar-title">DIY STREAM DECK V2</div>
      <nav class="top-nav" aria-label="Seiten">
        <button :class="['nav-btn', { active: currentPage === 'config' }]" type="button" @click="currentPage = 'config'">
          Konfigurator
        </button>
        <button :class="['nav-btn', { active: currentPage === 'test' }]" type="button" @click="currentPage = 'test'">
          Test
        </button>
        <button :class="['nav-btn', { active: currentPage === 'settings' }]" type="button" @click="currentPage = 'settings'">
          Settings
        </button>
      </nav>
      <div v-if="currentPage === 'config'" class="profile-selector">
        <label for="profile-select">Aktives Profil:</label>
        <select id="profile-select" v-model.number="currentProfileId" @change="clearSelection">
          <option :value="0">Main (Desktop)</option>
          <option :value="1">Gaming</option>
          <option :value="2">Streaming (OBS)</option>
        </select>
      </div>
    </header>

    <main v-if="currentPage === 'config'" class="app-container">
      <section class="main-content">
        <div class="preview-area">
          <div class="hardware-board">
            <div class="oled-display">
              <div class="oled-header">PROFIL: {{ currentProfile.name }}</div>
              <div class="oled-content">
                <div
                  v-for="(segment, idx) in oledSegments"
                  :key="`seg-${idx}`"
                  :class="['oled-segment', { selected: selectedEncoderIndex === idx, divider: idx > 0 }]"
                >
                  <div class="oled-icon" aria-hidden="true">
                    <div v-for="(row, rowIdx) in segment.icon" :key="`row-${idx}-${rowIdx}`" class="oled-icon-row">
                      <span v-for="(on, colIdx) in row" :key="`px-${idx}-${rowIdx}-${colIdx}`" :class="['oled-px', { on }]" />
                    </div>
                  </div>
                  <div class="oled-val">{{ segment.value }}%</div>
                </div>
              </div>
            </div>

            <div class="encoders-row">
              <button
                v-for="enc in encoderIds"
                :key="enc"
                :data-id="enc"
                :class="['encoder', { selected: selectedElementId === enc }]"
                :title="`Encoder ${extractIndex(enc) + 1}`"
                type="button"
                @click="selectElement(enc)"
              />
            </div>

            <div class="button-grid">
              <button
                v-for="btn in buttonIds"
                :key="btn"
                :data-id="btn"
                :class="['btn', { selected: selectedElementId === btn, 'drag-over': dragOverButtonId === btn }]"
                type="button"
                @click="selectElement(btn)"
                @dragover.prevent="onButtonDragOver(btn)"
                @dragenter.prevent="onButtonDragOver(btn)"
                @dragleave="onButtonDragLeave(btn)"
                @drop.prevent="onButtonDrop(btn)"
              >
                <span class="icon">{{ buttonIcon(btn) }}</span>
                <span class="label">{{ buttonLabel(btn) }}</span>
              </button>
            </div>
          </div>
        </div>

        <div class="config-area">
          <div v-if="showPlaceholder" class="config-placeholder">{{ placeholderText }}</div>

          <div v-if="showVolumeForm" class="config-form active">
            <div class="form-group">
              <label>Aktion</label>
              <input type="text" value="Lautstärke / Gain" disabled />
            </div>
            <div class="form-group">
              <label>Applikation</label>
              <select v-model="volumeAppModel">
                <option value="System">System Master</option>
                <option value="Spotify">Spotify</option>
                <option value="Discord">Discord</option>
                <option value="Game">Game Audio</option>
                <option value="Browser">Browser</option>
                <option value="Mic">Mikrofon</option>
              </select>
            </div>
            <div class="form-group">
              <label>Push-Button</label>
              <select>
                <option>Mute Toggle</option>
                <option>Reset 50%</option>
              </select>
            </div>
          </div>

          <div v-if="showAppForm" class="config-form active">
            <div class="form-group">
              <label>Aktion</label>
              <input :value="selectedActionLabel" type="text" disabled />
            </div>
            <div class="form-group">
              <label>Titel</label>
              <input v-model="appTitleModel" type="text" placeholder="Text auf dem Button" />
            </div>
          </div>
        </div>
      </section>

      <aside class="sidebar">
        <div class="action-list">
          <div class="action-category">Rotary Encoders</div>
          <button
            class="action-item"
            type="button"
            draggable="true"
            @click="assignAction('volume')"
            @dragstart="onActionDragStart('volume', $event)"
            @dragend="onActionDragEnd"
          >
            <span class="action-icon">&#128266;</span><span>Lautstärke Mixer</span>
          </button>

          <div class="action-category">Grid Buttons</div>
          <button
            class="action-item"
            type="button"
            draggable="true"
            @click="assignAction('launch')"
            @dragstart="onActionDragStart('launch', $event)"
            @dragend="onActionDragEnd"
          >
            <span class="action-icon">&#128640;</span><span>App Starten</span>
          </button>
          <button
            class="action-item"
            type="button"
            draggable="true"
            @click="assignAction('hotkey')"
            @dragstart="onActionDragStart('hotkey', $event)"
            @dragend="onActionDragEnd"
          >
            <span class="action-icon">&#9000;&#65039;</span><span>Tastatur Makro</span>
          </button>
          <button
            class="action-item"
            type="button"
            draggable="true"
            @click="assignAction('media')"
            @dragstart="onActionDragStart('media', $event)"
            @dragend="onActionDragEnd"
          >
            <span class="action-icon">&#9199;&#65039;</span><span>Media Control</span>
          </button>
          <button
            class="action-item"
            type="button"
            draggable="true"
            @click="assignAction('profile')"
            @dragstart="onActionDragStart('profile', $event)"
            @dragend="onActionDragEnd"
          >
            <span class="action-icon">&#128193;</span><span>Profil Wechseln</span>
          </button>
        </div>
      </aside>
    </main>

    <main v-else-if="currentPage === 'test'" class="test-page">
      <section class="test-layout">
        <div class="test-header">
          <h2>LED Testseite</h2>
          <p>Linksklick: Auslösen. Rechtsklick: Button Settings.</p>
        </div>

        <div class="test-grid">
          <button
            v-for="btn in testButtonConfigs"
            :key="btn.id"
            class="test-key"
            type="button"
            @click="triggerAction(btn.command)"
            @contextmenu.prevent="openTestConfig(btn)"
          >
            <span class="test-key-icon">{{ btn.icon }}</span>
            <span :class="['test-key-label', { danger: btn.command === 'StartBootloader' }]">{{ btn.label }}</span>
          </button>
        </div>

        <div class="test-log">
          <div class="test-log-title">System Log</div>
          <div v-if="picoLogs.length === 0" class="test-log-entry muted">Warte auf serielle Kommunikation...</div>
          <div v-for="(log, idx) in picoLogs" :key="`log-${idx}`" class="test-log-entry">{{ log }}</div>
        </div>
      </section>

      <aside :class="['test-settings', { open: isTestConfigOpen }]">
        <div class="test-settings-header">
          <h3>Button Settings</h3>
          <button type="button" class="close-btn" @click="isTestConfigOpen = false">✕</button>
        </div>

        <div class="test-settings-body">
          <div class="form-group block">
            <label>Beschriftung</label>
            <input v-model="editData.label" type="text" />
          </div>

          <div class="form-group block">
            <label>Icon (Emoji/Text)</label>
            <input v-model="editData.icon" type="text" />
          </div>

          <div class="form-group block">
            <label>Aktion / Effekt</label>
            <select v-model="editData.effectName">
              <option v-for="name in effectNames" :key="name" :value="name">{{ name }}</option>
            </select>
          </div>

          <div v-if="currentEffectNeeds.includes('color')" class="form-group block">
            <label>Farbe (Hex)</label>
            <input v-model="editData.color" type="text" placeholder="#FFFFFF" />
          </div>

          <div v-if="currentEffectNeeds.includes('brightness')" class="form-group block">
            <label>Helligkeit</label>
            <input v-model.number="editData.params.brightness" type="number" min="0" max="255" />
          </div>

          <div v-if="currentEffectNeeds.includes('speed')" class="form-group block">
            <label>Tempo</label>
            <input v-model.number="editData.params.speed" type="number" min="1" max="255" />
          </div>

          <div v-if="currentEffectNeeds.includes('size')" class="form-group block">
            <label>Größe</label>
            <input v-model.number="editData.params.size" type="number" min="1" max="20" />
          </div>

          <div v-if="currentEffectNeeds.includes('tail')" class="form-group block">
            <label>Schweif-Länge</label>
            <input v-model.number="editData.params.tail" type="number" min="1" max="20" />
          </div>

          <div v-if="currentEffectNeeds.includes('density')" class="form-group block">
            <label>Dichte</label>
            <input v-model.number="editData.params.density" type="number" min="1" max="255" />
          </div>

          <div v-if="currentEffectNeeds.includes('hue')" class="form-group block">
            <label>Hue</label>
            <input v-model.number="editData.params.hue" type="number" min="0" max="255" />
          </div>

          <div v-if="currentEffectNeeds.includes('hue_shift')" class="form-group block">
            <label>Hue Shift</label>
            <input v-model.number="editData.params.hue_shift" type="number" min="1" max="100" />
          </div>

          <div v-if="currentEffectNeeds.includes('saturation')" class="form-group block">
            <label>Sättigung</label>
            <input v-model.number="editData.params.saturation" type="number" min="0" max="255" />
          </div>

          <div v-if="currentEffectNeeds.includes('spread')" class="form-group block">
            <label>Spread</label>
            <input v-model.number="editData.params.spread" type="number" min="1" max="255" />
          </div>

          <button type="button" class="save-btn" @click="saveTestSettings">Speichern & Anwenden</button>
        </div>
      </aside>
    </main>

    <main v-else class="settings-page">
      <div class="settings-empty">Settings kommt als nächstes.</div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

type PageName = 'config' | 'test' | 'settings'
type ActionType = 'volume' | 'launch' | 'hotkey' | 'media' | 'profile'

interface KeyConfig {
  type: ActionType
  app?: string
  title?: string
  value?: number
}

interface Profile {
  name: string
  keys: Record<string, KeyConfig>
}

interface TestButtonConfig {
  id: number
  label: string
  icon: string
  command: any
}

interface TestEditData {
  label: string
  icon: string
  effectName: string
  color: string
  params: {
    brightness: number
    speed: number
    size: number
    tail: number
    density: number
    hue: number
    hue_shift: number
    saturation: number
    spread: number
  }
}

const currentPage = ref<PageName>('config')

const encoderIds = ['enc-0', 'enc-1', 'enc-2', 'enc-3']
const buttonIds = ['btn-0', 'btn-1', 'btn-2', 'btn-3', 'btn-4', 'btn-5', 'btn-6', 'btn-7']
const defaultVolumes = [50, 65, 80, 35]

const iconMaster = [
  '000000000000',
  '000011110000',
  '001110011100',
  '011000000110',
  '011001100110',
  '110011110011',
  '110011110011',
  '011001100110',
  '011000000110',
  '001110011100',
  '000011110000',
  '000000000000'
]

const iconSpotify = [
  '000000000000',
  '000011111000',
  '001111111110',
  '011111111111',
  '011000000011',
  '111111111111',
  '111100001111',
  '111111111111',
  '011110011110',
  '011111111111',
  '001111111110',
  '000011111000'
]

const iconDiscord = [
  '000000000000',
  '000110011000',
  '001111111100',
  '011111111110',
  '011011110110',
  '011011110110',
  '011111111110',
  '001110111100',
  '000110011000',
  '000000000000',
  '000000000000',
  '000000000000'
]

const iconBrowser = [
  '000000000000',
  '000111111000',
  '001101101100',
  '011001100110',
  '011001100110',
  '111111111111',
  '111111111111',
  '011001100110',
  '011001100110',
  '001101101100',
  '000111111000',
  '000000000000'
]

const iconMap: Record<string, string[]> = {
  system: iconMaster,
  spotify: iconSpotify,
  discord: iconDiscord,
  game: iconMaster,
  browser: iconBrowser,
  mic: iconDiscord
}

const profiles = reactive<Record<number, Profile>>({
  0: {
    name: 'MAIN',
    keys: {
      'enc-0': { type: 'volume', app: 'System', value: 50 },
      'enc-1': { type: 'volume', app: 'Spotify', value: 65 },
      'enc-2': { type: 'volume', app: 'Discord', value: 80 },
      'enc-3': { type: 'volume', app: 'Browser', value: 35 },
      'btn-0': { type: 'launch', title: 'Browser' },
      'btn-3': { type: 'profile', title: 'Next Profile' }
    }
  },
  1: {
    name: 'GAMING',
    keys: {
      'enc-0': { type: 'volume', app: 'Game', value: 50 },
      'enc-1': { type: 'volume', app: 'Discord', value: 65 },
      'enc-2': { type: 'volume', app: 'Mic', value: 80 },
      'enc-3': { type: 'volume', app: 'System', value: 35 },
      'btn-0': { type: 'launch', title: 'Steam' },
      'btn-3': { type: 'profile', title: 'Next Profile' }
    }
  },
  2: {
    name: 'STREAMING',
    keys: {
      'enc-0': { type: 'volume', app: 'System', value: 50 },
      'enc-1': { type: 'volume', app: 'Spotify', value: 65 },
      'enc-2': { type: 'volume', app: 'Discord', value: 80 },
      'enc-3': { type: 'volume', app: 'Mic', value: 35 },
      'btn-0': { type: 'hotkey', title: 'OBS Rec' },
      'btn-1': { type: 'hotkey', title: 'Mute All' },
      'btn-3': { type: 'profile', title: 'Next Profile' }
    }
  }
})

const currentProfileId = ref(0)
const selectedElementId = ref<string | null>(null)
const dragActionType = ref<ActionType | null>(null)
const dragOverButtonId = ref<string | null>(null)

const currentProfile = computed(() => profiles[currentProfileId.value])

const selectedEncoderIndex = computed(() => {
  if (!selectedElementId.value || !selectedElementId.value.startsWith('enc-')) return -1
  return extractIndex(selectedElementId.value)
})

const selectedData = computed(() => {
  if (!selectedElementId.value) return null
  return currentProfile.value.keys[selectedElementId.value] || null
})

const showVolumeForm = computed(() => !!selectedData.value && selectedData.value.type === 'volume')
const showAppForm = computed(() => !!selectedData.value && selectedData.value.type !== 'volume')
const showPlaceholder = computed(() => !showVolumeForm.value && !showAppForm.value)

const placeholderText = computed(() => {
  if (!selectedElementId.value) {
    return 'Wähle eine Taste oder einen Encoder im aktuellen Profil, um Aktionen zu konfigurieren.'
  }
  if (!selectedData.value) {
    return 'Leer. Weise rechts eine Aktion zu.'
  }
  return ''
})

const selectedActionLabel = computed(() => {
  if (!selectedData.value) return 'Aktion'
  const labels: Record<ActionType, string> = {
    launch: 'App Starten',
    hotkey: 'Makro',
    media: 'Media Control',
    profile: 'Profil wechseln',
    volume: 'Lautstärke / Gain'
  }
  return labels[selectedData.value.type]
})

const volumeAppModel = computed({
  get: () => {
    if (!showVolumeForm.value || !selectedData.value) return 'System'
    return selectedData.value.app || 'System'
  },
  set: (value: string) => {
    if (!selectedElementId.value) return
    const key = currentProfile.value.keys[selectedElementId.value]
    if (!key || key.type !== 'volume') return
    key.app = value
  }
})

const appTitleModel = computed({
  get: () => {
    if (!showAppForm.value || !selectedData.value) return ''
    return selectedData.value.title || ''
  },
  set: (value: string) => {
    if (!selectedElementId.value) return
    const key = currentProfile.value.keys[selectedElementId.value]
    if (!key || key.type === 'volume') return
    key.title = value
  }
})

const oledSegments = computed(() =>
  encoderIds.map((id, idx) => {
    const key = currentProfile.value.keys[id]
    const app = key?.app || 'System'
    const icon = iconMap[app.toLowerCase()] || iconMaster
    const value = key?.value ?? defaultVolumes[idx]
    return { icon: toMatrix(icon), value }
  })
)

function clearSelection() {
  selectedElementId.value = null
}

function selectElement(id: string) {
  selectedElementId.value = id
}

function extractIndex(id: string): number {
  const index = Number(id.split('-')[1])
  return Number.isNaN(index) ? 0 : index
}

function toMatrix(icon: string[]): boolean[][] {
  return icon.map((line) => Array.from(line).map((pixel) => pixel === '1'))
}

function assignAction(actionType: ActionType, targetId?: string) {
  const id = targetId ?? selectedElementId.value
  if (!id) {
    window.alert('Bitte wähle zuerst einen Button im Zentrum aus.')
    return
  }

  const next: KeyConfig = { type: actionType }
  if (actionType === 'volume') {
    const index = extractIndex(id)
    next.app = 'System'
    next.value = defaultVolumes[index] ?? 50
  } else if (actionType === 'launch') {
    next.title = 'App'
  } else if (actionType === 'profile') {
    next.title = 'Switch'
  }
  currentProfile.value.keys[id] = next
}

function onActionDragStart(actionType: ActionType, event: DragEvent) {
  dragActionType.value = actionType
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'copy'
    event.dataTransfer.setData('text/plain', actionType)
  }
}

function onActionDragEnd() {
  dragActionType.value = null
  dragOverButtonId.value = null
}

function onButtonDragOver(buttonId: string) {
  if (!dragActionType.value) return
  dragOverButtonId.value = buttonId
}

function onButtonDragLeave(buttonId: string) {
  if (dragOverButtonId.value === buttonId) {
    dragOverButtonId.value = null
  }
}

function onButtonDrop(buttonId: string) {
  if (!dragActionType.value) return
  assignAction(dragActionType.value, buttonId)
  selectedElementId.value = buttonId
  dragActionType.value = null
  dragOverButtonId.value = null
}

function buttonIcon(id: string) {
  const data = currentProfile.value.keys[id]
  if (!data) return ''
  if (data.type === 'launch') return '🚀'
  if (data.type === 'media') return '⏯️'
  if (data.type === 'hotkey') return '⌨️'
  if (data.type === 'profile') return '📁'
  return ''
}

function buttonLabel(id: string) {
  const data = currentProfile.value.keys[id]
  if (!data) return ''
  if (data.type === 'media') return 'Media'
  return data.title || ''
}

const picoLogs = ref<string[]>([])

const testButtonConfigs = ref<TestButtonConfig[]>([
  { id: 1, label: 'Solid White', icon: '💡', command: { FillAll: { r: 255, g: 255, b: 255, brightness: 50 } } },
  { id: 2, label: 'Blink Red', icon: '🚨', command: { SetEffect: { effect: { Blink: { r: 255, g: 0, b: 0, brightness: 50, speed: 200 } } } } },
  { id: 3, label: 'Rainbow', icon: '🎨', command: { SetEffect: { effect: { Rainbow: { speed: 15, brightness: 40 } } } } },
  { id: 4, label: 'Breathe Blue', icon: '🌬️', command: { SetEffect: { effect: { Breathing: { r: 0, g: 150, b: 255, brightness: 50, speed: 5 } } } } },
  { id: 5, label: 'Chase Green', icon: '🏃', command: { SetEffect: { effect: { Chase: { r: 0, g: 255, b: 0, brightness: 50, speed: 80, size: 3 } } } } },
  { id: 6, label: 'Comet Fire', icon: '🔥', command: { SetEffect: { effect: { Comet: { r: 255, g: 100, b: 0, brightness: 50, speed: 60, tail: 8 } } } } },
  { id: 7, label: 'Sparkle', icon: '✨', command: { SetEffect: { effect: { Sparkle: { r: 255, g: 255, b: 255, brightness: 50, speed: 150, density: 50 } } } } },
  { id: 8, label: 'Aurora', icon: '🌊', command: { SetEffect: { effect: { Aurora: { speed: 10, brightness: 40 } } } } },
  { id: 9, label: 'Orbit', icon: '🔄', command: { SetEffect: { effect: { ColorOrbit: { hue: 140, hue_shift: 16, saturation: 220, brightness: 50, speed: 35 } } } } },
  { id: 10, label: 'Astolfo', icon: '💗', command: { SetEffect: { effect: { Astolfo: { brightness: 50, speed: 190, saturation: 220, spread: 90 } } } } },
  { id: 11, label: 'Blackout', icon: '⏻', command: { FillAll: { r: 0, g: 0, b: 0, brightness: 0 } } },
  { id: 12, label: 'Flash Mode', icon: '📲', command: 'StartBootloader' }
])

const effectRequirements: Record<string, string[]> = {
  FillAll: ['color', 'brightness'],
  Solid: ['color', 'brightness'],
  Blink: ['color', 'brightness', 'speed'],
  Rainbow: ['brightness', 'speed'],
  Breathing: ['color', 'brightness', 'speed'],
  Chase: ['color', 'brightness', 'speed', 'size'],
  Comet: ['color', 'brightness', 'speed', 'tail'],
  Sparkle: ['color', 'brightness', 'speed', 'density'],
  Aurora: ['brightness', 'speed'],
  ColorOrbit: ['hue', 'hue_shift', 'saturation', 'brightness', 'speed'],
  Astolfo: ['brightness', 'speed', 'saturation', 'spread'],
  Ping: [],
  StartBootloader: []
}

const effectNames = Object.keys(effectRequirements)
const isTestConfigOpen = ref(false)
const editingTestId = ref<number | null>(null)

const editData = ref<TestEditData>({
  label: '',
  icon: '',
  effectName: 'FillAll',
  color: '#FFFFFF',
  params: {
    brightness: 50,
    speed: 100,
    size: 3,
    tail: 5,
    density: 128,
    hue: 140,
    hue_shift: 16,
    saturation: 220,
    spread: 90
  }
})

const currentEffectNeeds = computed(() => effectRequirements[editData.value.effectName] || [])

const rgbToHex = (r: number, g: number, b: number) => `#${(1 << 24 | (r << 16) | (g << 8) | b).toString(16).slice(1).toUpperCase()}`
const hexToRgb = (hex: string) => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex.trim())
  return result ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) } : { r: 255, g: 255, b: 255 }
}

function openTestConfig(btn: TestButtonConfig) {
  editingTestId.value = btn.id
  editData.value.label = btn.label
  editData.value.icon = btn.icon
  const cmd = btn.command

  if (cmd === 'Ping' || cmd === 'StartBootloader') {
    editData.value.effectName = cmd
  } else if (cmd?.FillAll) {
    editData.value.effectName = 'FillAll'
    editData.value.color = rgbToHex(cmd.FillAll.r, cmd.FillAll.g, cmd.FillAll.b)
    editData.value.params.brightness = cmd.FillAll.brightness || 50
  } else if (cmd?.SetEffect?.effect) {
    const effectName = Object.keys(cmd.SetEffect.effect)[0]
    editData.value.effectName = effectName
    const payload = cmd.SetEffect.effect[effectName]
    if (payload.r !== undefined) {
      editData.value.color = rgbToHex(payload.r, payload.g, payload.b)
    }
    Object.assign(editData.value.params, payload)
  }
  isTestConfigOpen.value = true
}

async function saveTestSettings() {
  const btnIndex = testButtonConfigs.value.findIndex((btn) => btn.id === editingTestId.value)
  if (btnIndex === -1) return

  const needs = currentEffectNeeds.value
  const p = editData.value.params
  const rgb = hexToRgb(editData.value.color)
  const effectName = editData.value.effectName

  let newCmd: any = {}
  if (effectName === 'Ping' || effectName === 'StartBootloader') {
    newCmd = effectName
  } else if (effectName === 'FillAll') {
    newCmd = { FillAll: { r: Math.round(rgb.r), g: Math.round(rgb.g), b: Math.round(rgb.b), brightness: Math.round(p.brightness) } }
  } else {
    const payload: any = {}
    if (needs.includes('color')) {
      payload.r = Math.round(rgb.r)
      payload.g = Math.round(rgb.g)
      payload.b = Math.round(rgb.b)
    }
    if (needs.includes('brightness')) payload.brightness = Math.round(p.brightness)
    if (needs.includes('speed')) payload.speed = Math.round(p.speed)
    if (needs.includes('size')) payload.size = Math.round(p.size)
    if (needs.includes('tail')) payload.tail = Math.round(p.tail)
    if (needs.includes('density')) payload.density = Math.round(p.density)
    if (needs.includes('hue')) payload.hue = Math.round(p.hue)
    if (needs.includes('hue_shift')) payload.hue_shift = Math.round(p.hue_shift)
    if (needs.includes('saturation')) payload.saturation = Math.round(p.saturation)
    if (needs.includes('spread')) payload.spread = Math.round(p.spread)
    newCmd = { SetEffect: { effect: { [effectName]: payload } } }
  }

  testButtonConfigs.value[btnIndex].label = editData.value.label
  testButtonConfigs.value[btnIndex].icon = editData.value.icon
  testButtonConfigs.value[btnIndex].command = newCmd
  isTestConfigOpen.value = false
  await triggerAction(newCmd)
}

async function triggerAction(command: any) {
  try {
    await invoke('send_to_pico', { command })
    addLog(`CMD: ${typeof command === 'string' ? command : Object.keys(command)[0]}`)
  } catch (error) {
    addLog(`ERR: ${String(error)}`)
  }
}

function addLog(msg: string) {
  picoLogs.value.unshift(msg)
  if (picoLogs.value.length > 10) picoLogs.value.pop()
}
</script>

<style scoped>
.app-root {
  --bg-main: #181818;
  --bg-panel-shared: #2d2d2d;
  --bg-hover: #333333;
  --border-color: #454545;
  --divider-light: #767676;
  --text-main: #f0f0f0;
  --text-muted: #a0a0a0;
  --accent: #0078d7;
  --accent-glow: rgba(0, 120, 215, 0.4);
  --oled-bg: #050505;
  --oled-pixel: #d1d1d1;

  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
  background-color: var(--bg-main);
  color: var(--text-main);
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

* {
  box-sizing: border-box;
}

.top-bar {
  height: 56px;
  background-color: #111;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 0 20px;
}

.top-bar-title {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 1px;
  white-space: nowrap;
}

.top-nav {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.nav-btn {
  border: 1px solid #464646;
  background: #242424;
  color: #d8d8d8;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
}

.nav-btn.active {
  border-color: #bdbdbd;
  background: #333;
  color: #fff;
}

.profile-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.profile-selector label {
  font-size: 12px;
  color: #aaa;
}

.profile-selector select {
  background-color: #2a2a2a;
  color: white;
  border: 1px solid #444;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 14px;
  outline: none;
  cursor: pointer;
}

.app-container {
  display: flex;
  flex: 1;
  min-height: 0;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--divider-light);
  min-width: 0;
}

.preview-area {
  flex: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--bg-main);
  padding: 24px;
}

.hardware-board {
  background: #1e1e1e;
  padding: 40px;
  border-radius: 20px;
  box-shadow: 0 15px 40px rgba(0, 0, 0, 0.8), inset 0 1px 1px rgba(255, 255, 255, 0.05);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 30px;
  border: 1px solid #2a2a2a;
}

.oled-display {
  width: 256px;
  height: 128px;
  background-color: var(--oled-bg);
  border: 3px solid #111;
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  color: var(--oled-pixel);
  font-family: 'Courier New', Courier, monospace;
  box-shadow: inset 0 0 15px rgba(255, 255, 255, 0.05), 0 0 5px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  text-transform: uppercase;
}

.oled-header {
  height: 25%;
  border-bottom: 1px dashed #333;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 1px;
  background: repeating-linear-gradient(0deg, transparent, transparent 1px, rgba(0, 0, 0, 0.2) 1px, rgba(0, 0, 0, 0.2) 2px);
}

.oled-content {
  flex: 1;
  display: flex;
}

.oled-segment {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding-top: 6px;
}

.oled-segment.divider {
  border-left: 1px dashed #3a3a3a;
}

.oled-segment.selected {
  background: var(--oled-pixel);
  color: #050505;
}

.oled-icon {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.oled-icon-row {
  display: flex;
  gap: 1px;
}

.oled-px {
  width: 2px;
  height: 2px;
  opacity: 0.1;
  background: currentColor;
}

.oled-px.on {
  opacity: 1;
}

.oled-val {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 1px;
}

.encoders-row {
  display: flex;
  gap: 20px;
  width: 256px;
  justify-content: space-between;
}

.encoder {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: linear-gradient(135deg, #444, #1a1a1a);
  border: 2px solid #111;
  box-shadow: 0 5px 10px rgba(0, 0, 0, 0.5), inset 0 2px 2px rgba(255, 255, 255, 0.2);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.encoder::after {
  content: '';
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #252525;
  border: 1px solid #111;
}

.encoder.selected,
.btn.selected {
  border-color: var(--accent);
  box-shadow: 0 0 15px var(--accent-glow);
}

.button-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-top: 10px;
}

.btn {
  width: 70px;
  height: 70px;
  background: #151515;
  border: 2px solid #2a2a2a;
  border-radius: 14px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--text-muted);
  transition: all 0.2s;
  overflow: hidden;
  text-align: center;
  padding: 5px;
}

.btn:hover {
  border-color: #555;
  background: #1a1a1a;
}

.btn.drag-over {
  border-color: #d5d5d5;
  box-shadow: 0 0 0 2px rgba(213, 213, 213, 0.35);
}

.btn .icon {
  font-size: 22px;
  margin-bottom: 4px;
  line-height: 1;
}

.config-area {
  flex: 1;
  background-color: var(--bg-panel-shared);
  border-top: 1px solid var(--divider-light);
  padding: 30px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.config-placeholder {
  color: var(--text-muted);
  margin-top: 20px;
}

.config-form {
  width: 100%;
  max-width: 500px;
}

.form-group {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
}

.form-group.block {
  margin-bottom: 14px;
  flex-direction: column;
  align-items: stretch;
  gap: 6px;
}

.form-group label {
  width: 120px;
  color: var(--text-muted);
  font-size: 13px;
  text-align: right;
  padding-right: 15px;
}

.form-group.block label {
  width: 100%;
  text-align: left;
  padding-right: 0;
}

.form-group input,
.form-group select {
  flex: 1;
  padding: 8px 12px;
  background-color: #1e1e1e;
  border: 1px solid var(--border-color);
  color: var(--text-main);
  border-radius: 4px;
  outline: none;
  font-size: 14px;
}

.sidebar {
  width: 280px;
  background-color: var(--bg-panel-shared);
  display: flex;
  flex-direction: column;
}

.action-list {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 16px;
}

.action-category {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  padding: 15px 15px 5px;
  font-weight: 700;
}

.action-item {
  width: 100%;
  background: transparent;
  border: 0;
  color: inherit;
  text-align: left;
  padding: 10px 15px 10px 30px;
  cursor: grab;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  transition: background 0.2s;
}

.action-item:hover {
  background-color: var(--bg-hover);
}

.test-page {
  flex: 1;
  display: flex;
  background: #141414;
  min-height: 0;
}

.test-layout {
  flex: 1;
  padding: 24px;
  overflow: auto;
}

.test-header h2 {
  font-size: 22px;
  margin: 0 0 6px;
}

.test-header p {
  margin: 0;
  color: #9b9b9b;
  font-size: 13px;
}

.test-grid {
  margin-top: 18px;
  display: grid;
  grid-template-columns: repeat(4, minmax(120px, 1fr));
  gap: 14px;
}

.test-key {
  min-height: 110px;
  border: 1px solid #343434;
  background: linear-gradient(145deg, #1f1f1f, #171717);
  border-radius: 14px;
  color: #f0f0f0;
  padding: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
}

.test-key:hover {
  border-color: #4d4d4d;
}

.test-key-icon {
  font-size: 26px;
}

.test-key-label {
  font-size: 12px;
  text-transform: uppercase;
  text-align: center;
  color: #bdbdbd;
  font-weight: 700;
}

.test-key-label.danger {
  color: #ff7d7d;
}

.test-log {
  margin-top: 20px;
  border: 1px solid #2f2f2f;
  background: #0c0c0c;
  border-radius: 10px;
  padding: 12px;
  min-height: 150px;
}

.test-log-title {
  font-size: 12px;
  text-transform: uppercase;
  color: #8e8e8e;
  margin-bottom: 10px;
}

.test-log-entry {
  font-family: 'Consolas', 'Courier New', monospace;
  color: #9dd29d;
  font-size: 13px;
  margin-bottom: 6px;
}

.test-log-entry.muted {
  color: #7f7f7f;
}

.test-settings {
  width: 0;
  border-left: 1px solid transparent;
  background: #1d1d1d;
  overflow: hidden;
  transition: width 0.2s ease;
}

.test-settings.open {
  width: 340px;
  border-left-color: #525252;
}

.test-settings-header {
  padding: 14px;
  border-bottom: 1px solid #3f3f3f;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.test-settings-header h3 {
  margin: 0;
  font-size: 16px;
}

.close-btn {
  border: 1px solid #525252;
  background: #2a2a2a;
  color: #ddd;
  border-radius: 6px;
  padding: 4px 8px;
  cursor: pointer;
}

.test-settings-body {
  padding: 14px;
}

.save-btn {
  width: 100%;
  border: 1px solid #5d5d5d;
  background: #2f2f2f;
  color: #f0f0f0;
  border-radius: 6px;
  padding: 10px;
  cursor: pointer;
  margin-top: 10px;
}

.settings-page {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #171717;
}

.settings-empty {
  color: #9a9a9a;
  font-size: 18px;
}

@media (max-width: 1200px) {
  .top-bar {
    flex-wrap: wrap;
    height: auto;
    padding: 10px 12px;
  }

  .profile-selector {
    width: 100%;
    justify-content: flex-end;
  }

  .app-container {
    flex-direction: column;
    overflow: auto;
  }

  .main-content {
    border-right: 0;
    border-bottom: 1px solid var(--divider-light);
  }

  .sidebar {
    width: 100%;
  }

  .test-page {
    flex-direction: column;
  }

  .test-settings,
  .test-settings.open {
    width: 100%;
    border-left: 0;
    border-top: 1px solid #525252;
  }
}

@media (max-width: 700px) {
  .hardware-board {
    transform: scale(0.92);
    transform-origin: top center;
    padding: 20px;
  }

  .config-area {
    padding: 18px;
  }

  .form-group {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .form-group label {
    width: 100%;
    text-align: left;
    padding-right: 0;
  }

  .test-grid {
    grid-template-columns: repeat(2, minmax(120px, 1fr));
  }
}
</style>
