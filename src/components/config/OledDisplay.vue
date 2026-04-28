<template>
  <div
      class="oled-container"
      :class="{ 'is-selected': store.selectedElementId === 'oled-display' }"
      @click="store.selectElement('oled-display')"
  >
    <div class="oled-wrapper">
      <canvas
          ref="oledCanvas"
          width="128"
          height="64"
          class="oled-canvas"
      ></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watchEffect } from 'vue';
import { useStreamDeckStore } from '@/stores/streamdeck';
import { listen } from '@tauri-apps/api/event';

const store = useStreamDeckStore();
const oledCanvas = ref<HTMLCanvasElement | null>(null);
let unlistenAudioUpdate: (() => void) | null = null;

// Fallback-Daten
const VOLUMES = [50, 65, 80, 35];
const defaultIcons = ['MASTER', 'SPOTIFY', 'DISCORD', 'BROWSER','MIC','CAMERA','PLAY_PAUSE', 'LIGHT', 'ACTIVE_WINDOW', 'NONE'];

const ICONS: Record<string, string[]> = {
  MASTER: [
    "              ", "       11     ", "      111     ", "     1111   1 ",
    "   111111   11", "   111111  111", "   111111  111", "   111111  111",
    "   111111  111", "   111111   11", "     1111   1 ", "      111     ",
    "       11     ", "              "
  ],
  SPOTIFY: [
    "            ", "    111111   ", "  1111111111 ", " 111111111111",
    " 11        11", "11111111111111", "1111      1111", "11111111111111",
    " 1111    1111", " 111111111111", "  1111111111 ", "    111111   ",
    "            ", "            "
  ],
  DISCORD: [
    "            ", "            ", "            ", "   11  11   ",
    "  11111111  ", " 1111111111 ", " 11 1111 11 ", " 11 1111 11 ",
    " 1111111111 ", "  111  111  ", "   11  11   ", "            ",
    "            ", "            "
  ],
  BROWSER: [
    "              ", "     11111    ", "   111   111  ", "  111     111 ",
    " 111       111", " 111   1111111", " 111   1111111", " 111          ",
    " 111          ", "  111     11  ", "   111111111  ", "     11111    ",
    "              ", "              "
  ],
  MIC: [
    "              ",
    "     1111     ",
    "    111111    ",
    "    111111    ",
    "    111111    ",
    "    111111    ",
    "     1111     ",
    "   11    11   ",
    "   11111111   ",
    "      11      ",
    "      11      ",
    "    111111    ",
    "              ",
    "              ",
  ],
  CAMERA: [
    "              ",
    "              ",
    "              ",
    "  1111111  11 ",
    "  1111111 111 ",
    "  11111111111 ",
    "  11111111111 ",
    "  1111111 111 ",
    "  1111111  11 ",
    "              ",
    "              ",
    "              ",
    "              ",
    "              ",
  ],
  PLAY_PAUSE: [
    "              ",
    "              ",
    "  111     11  ",
    "  1111    11  ",
    "  11111   11  ",
    "  111111  11  ",
    "  1111111 11  ",
    "  111111  11  ",
    "  11111   11  ",
    "  1111    11  ",
    "  111     11  ",
    "              ",
    "              ",
    "              ",
  ],
  LIGHT: [
    "              ",
    "     1111     ",
    "   11111111   ",
    "  1111  1111  ",
    "  111    111  ",
    "  111    111  ",
    "   111  111   ",
    "    111111    ",
    "    111111    ",
    "     1111     ",
    "     1111     ",
    "      11      ",
    "              ",
    "              ",
  ],
  ACTIVE_WINDOW: [
    "              ",
    "  11111111    ",
    "  11111111    ",
    "  1      1    ",
    "  1   11111111",
    "  1   11111111",
    "  11111      1",
    "      1      1",
    "      1      1",
    "      1      1",
    "      11111111",
    "              ",
    "              ",
    "              ",
  ],
  NONE: [""]
};

const FONT_5X7: Record<string, number[]> = {
  // ... (dein Font Code) ...
  ' ': [0x00, 0x00, 0x00, 0x00, 0x00],
  '%': [0x23, 0x13, 0x08, 0x64, 0x62],
  '-': [0x08, 0x08, 0x08, 0x08, 0x08],
  ':': [0x00, 0x00, 0x24, 0x00, 0x00],
  '0': [0x3e, 0x51, 0x49, 0x45, 0x3e],
  '1': [0x00, 0x42, 0x7f, 0x40, 0x00],
  '2': [0x42, 0x61, 0x51, 0x49, 0x46],
  '3': [0x21, 0x41, 0x45, 0x4b, 0x31],
  '4': [0x18, 0x14, 0x12, 0x7f, 0x10],
  '5': [0x27, 0x45, 0x45, 0x45, 0x39],
  '6': [0x3c, 0x4a, 0x49, 0x49, 0x30],
  '7': [0x01, 0x71, 0x09, 0x05, 0x03],
  '8': [0x36, 0x49, 0x49, 0x49, 0x36],
  '9': [0x06, 0x49, 0x49, 0x29, 0x1e],
  'A': [0x7e, 0x11, 0x11, 0x11, 0x7e],
  'B': [0x7f, 0x49, 0x49, 0x49, 0x36],
  'C': [0x3e, 0x41, 0x41, 0x41, 0x22],
  'D': [0x7f, 0x41, 0x41, 0x22, 0x1c],
  'E': [0x7f, 0x49, 0x49, 0x49, 0x41],
  'F': [0x7f, 0x09, 0x09, 0x09, 0x01],
  'G': [0x3e, 0x41, 0x49, 0x49, 0x7a],
  'H': [0x7f, 0x08, 0x08, 0x08, 0x7f],
  'I': [0x00, 0x41, 0x7f, 0x41, 0x00],
  'J': [0x20, 0x40, 0x41, 0x3f, 0x01],
  'K': [0x7f, 0x08, 0x14, 0x22, 0x41],
  'L': [0x7f, 0x40, 0x40, 0x40, 0x40],
  'M': [0x7f, 0x02, 0x04, 0x02, 0x7f],
  'N': [0x7f, 0x04, 0x08, 0x10, 0x7f],
  'O': [0x3e, 0x41, 0x41, 0x41, 0x3e],
  'P': [0x7f, 0x09, 0x09, 0x09, 0x06],
  'Q': [0x3e, 0x41, 0x51, 0x21, 0x5e],
  'R': [0x7f, 0x09, 0x19, 0x29, 0x46],
  'S': [0x46, 0x49, 0x49, 0x49, 0x31],
  'T': [0x01, 0x01, 0x7f, 0x01, 0x01],
  'U': [0x3f, 0x40, 0x40, 0x40, 0x3f],
  'V': [0x1f, 0x20, 0x40, 0x20, 0x1f],
  'W': [0x3f, 0x40, 0x38, 0x40, 0x3f],
  'X': [0x63, 0x14, 0x08, 0x14, 0x63],
  'Y': [0x03, 0x04, 0x78, 0x04, 0x03],
  'Z': [0x61, 0x51, 0x49, 0x45, 0x43],
};
const FONT_DEFAULT = [0x00, 0x00, 0x5f, 0x00, 0x00];

const renderDisplay = () => {
  if (!oledCanvas.value) return;
  const ctx = oledCanvas.value.getContext('2d');
  if (!ctx) return;

  const DISPLAY_WIDTH = 128;
  const DISPLAY_HEIGHT = 64;
  const buffer = new Uint8ClampedArray(DISPLAY_WIDTH * DISPLAY_HEIGHT * 4);

  const putPixel = (x: number, y: number, on: boolean) => {
    if (x < 0 || x >= DISPLAY_WIDTH || y < 0 || y >= DISPLAY_HEIGHT) return;
    const index = (y * DISPLAY_WIDTH + x) * 4;
    const color = on ? 255 : 0;
    buffer[index] = color;
    buffer[index + 1] = color;
    buffer[index + 2] = color;
    buffer[index + 3] = 255;
  };

  for (let i = 0; i < buffer.length; i += 4) {
    buffer[i] = 0; buffer[i+1] = 0; buffer[i+2] = 0; buffer[i+3] = 255;
  }

  const drawText = (col: number, page: number, text: string, on: boolean) => {
    let cursor = col;
    for (let i = 0; i < text.length; i++) {
      if (cursor + 6 > DISPLAY_WIDTH) break;
      const char = text[i].toUpperCase();
      const glyph = FONT_5X7[char] || FONT_DEFAULT;

      for (let dx = 0; dx < glyph.length; dx++) {
        const bits = glyph[dx];
        for (let dy = 0; dy < 7; dy++) {
          if ((bits >> dy) & 1) {
            putPixel(cursor + dx, page * 8 + dy, on);
          }
        }
      }
      cursor += 6;
    }
  };

  const drawTextCenteredInRange = (page: number, text: string, xMin: number, xMax: number, on: boolean) => {
    const glyphW = 6;
    const textW = text.length * glyphW;
    const rangeW = Math.max(0, xMax - xMin + 1);
    const startX = xMin + Math.floor(Math.max(0, rangeW - textW) / 2);
    drawText(startX, page, text, on);
  };

  const drawDashedHLine = (y: number, xStart: number, xEnd: number, dashLen: number, on: boolean) => {
    let x = xStart;
    while (x <= xEnd) {
      for (let d = 0; d < dashLen; d++) {
        if (x + d <= xEnd) putPixel(x + d, y, on);
      }
      x += dashLen * 2;
    }
  };

  const drawDashedVLine = (x: number, yStart: number, yEnd: number, dashLen: number, gapLen: number, on: boolean) => {
    let y = yStart;
    while (y <= yEnd) {
      for (let d = 0; d < dashLen; d++) {
        if (y + d <= yEnd) putPixel(x, y + d, on);
      }
      y += dashLen + gapLen;
    }
  };

  const drawIcon = (x: number, y: number, iconData: string[], on: boolean) => {
    for (let row = 0; row < iconData.length; row++) {
      const line = iconData[row];
      // HIER GEFIXT: Fehlende Zeichen werden dynamisch ausgeglichen (Icon zentriert sich selbst)
      const xOffset = Math.floor((14 - line.length) / 2);
      for (let col = 0; col < line.length; col++) {
        if (line[col] === '1') {
          putPixel(x + xOffset + col, y + row, on);
        }
      }
    }
  };

  const profileName = store.activeProfile?.name || 'MAIN';
  drawTextCenteredInRange(0, profileName.toUpperCase(), 0, DISPLAY_WIDTH - 1, true);
  drawDashedHLine(10, 0, DISPLAY_WIDTH - 1, 2, true);

  const segmentWidth = Math.floor(DISPLAY_WIDTH / 4);
  const displayConfig = store.activeProfile?.keys['oled-display']?.slots || [];

  for (let i = 0; i < 4; i++) {
    const xStart = i * segmentWidth;
    const iconX = xStart + 9;
    const iconY = 20;

    if (i > 0) drawDashedVLine(xStart, 15, DISPLAY_HEIGHT - 1, 1, 2, true);

    const slotData = displayConfig[i] || {};
    const iconKey = slotData.icon || defaultIcons[i];
    const iconData = ICONS[iconKey] || ICONS['MASTER'];

    drawIcon(iconX, iconY, iconData, true);

    const isMuted = slotData.muted ?? false;
    if (isMuted) {
      for (let d = 0; d < 14; d++) {
        putPixel(iconX + d, iconY + d, true);
        if (d < 13) putPixel(iconX + d + 1, iconY + d, true);
        putPixel(iconX + 13 - d, iconY + d, true);
        if (d > 0) putPixel(iconX + 13 - d - 1, iconY + d, true);
      }
    }

    const volume = slotData.value ?? VOLUMES[i];
    let volStr = "---";
    if (volume !== 255) {
      volStr = `${volume}%`;
    }

    drawTextCenteredInRange(6, volStr, xStart, xStart + segmentWidth - 1, true);
  }

  const imageData = new ImageData(buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT);
  ctx.putImageData(imageData, 0, 0);
};

onMounted(async () => {
  renderDisplay();

  // HIER GEFIXT: Frontend lauscht jetzt auf Events aus Rust
  unlistenAudioUpdate = await listen('audio-update', (event: any) => {
    const { slot, volume, muted } = event.payload;

    if (store.activeProfile) {
      if (!store.activeProfile.keys['oled-display']) {
        store.activeProfile.keys['oled-display'] = { slots: [] };
      }
      const slots = store.activeProfile.keys['oled-display'].slots || [];

      // Auffüllen, falls der Array noch zu klein ist
      while (slots.length <= slot) {
        slots.push({ icon: 'NONE', process: '' });
      }

      // Werte setzen
      slots[slot].value = volume;
      slots[slot].muted = muted;

      // Neues Array zuweisen, damit Vue die Änderung garantiert registriert
      store.activeProfile.keys['oled-display'].slots = [...slots];
    }
  });
});

onUnmounted(() => {
  if (unlistenAudioUpdate) unlistenAudioUpdate();
});

watchEffect(() => {
  renderDisplay();
});
</script>

<style scoped>
.oled-container {
  display: inline-block;
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  transition: all 0.2s ease;
  border: 2px solid transparent;
}

.oled-container:hover {
  background: rgba(255, 255, 255, 0.05);
}

.oled-container.is-selected {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.oled-wrapper {
  display: inline-block;
}

.oled-canvas {
  width: 256px;
  height: 128px;
  background-color: #000;
  border: 4px solid #333;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  display: block;
}
</style>