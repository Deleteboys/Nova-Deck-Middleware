import { ref, onMounted, onUnmounted } from 'vue';

export function useLedAnimation(getLedConfig: () => any) {
    const leftGrad = ref('');
    const bottomGrad = ref('');
    const rightGrad = ref('');

    const NUM_LEDS = 13;
    let rafId: number | null = null;

    // --- MATH HELPERS (Mirroring Firmware) ---
    const speed_step = (speed: number) => 1 + Math.floor((speed * 7) / 255);
    const blink_period_frames = (speed: number) => 6 + Math.floor(((255 - speed) * 54) / 255);
    const chase_period_frames = (speed: number) => 2 + Math.floor(((255 - speed) * 78) / 255);
    const comet_period_frames = (speed: number) => 1 + Math.floor(((255 - speed) * 39) / 255);
    const orbit_period_frames = (speed: number) => 1 + Math.floor(((255 - speed) * 18) / 255);

    const scale = (component: number, brightness: number) => Math.floor((component * brightness) / 255);
    const smoothstep8 = (x: number) => {
        let x32 = x;
        return Math.floor((x32 * x32 * (765 - 2 * x32)) / 65025);
    };
    const smooth_wave8 = (phase: number) => {
        let tri = phase < 128 ? phase * 2 : (255 - phase) * 2;
        return smoothstep8(tri);
    };
    const lerp8 = (a: number, b: number, t: number) => Math.floor(a + ((b - a) * t) / 255);

    const hsv_to_rgb = (h: number, s: number, v: number) => {
        if (s === 0) return { r: v, g: v, b: v };
        let region = Math.floor(h / 43);
        let remainder = Math.floor((h - (region * 43)) * 6);
        let p = Math.floor((v * (255 - s)) / 255);
        let q = Math.floor((v * (255 - Math.floor((s * remainder) / 255))) / 255);
        let t = Math.floor((v * (255 - Math.floor((s * (255 - remainder)) / 255))) / 255);
        switch (region) {
            case 0: return { r: v, g: t, b: p }; case 1: return { r: q, g: v, b: p };
            case 2: return { r: p, g: v, b: t }; case 3: return { r: p, g: q, b: v };
            case 4: return { r: t, g: p, b: v }; default: return { r: v, g: p, b: q };
        }
    };

    const applyBrightness = (r: number, g: number, b: number, brightness: number) => {
        // Realistische Gamma-Korrektur für die Web-Simulation
        const gamma = Math.pow(brightness / 255, 1 / 1.8);
        return { r: Math.floor(r * gamma), g: Math.floor(g * gamma), b: Math.floor(b * gamma) };
    };

    const hexToRgb = (hex: string) => {
        let result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) } : { r: 0, g: 0, b: 0 };
    };

    const renderLoop = (time: number) => {
        // Wandelt Zeit in "virtuelle Frames" um (basierend auf 60 FPS für Konsistenz mit Firmware-Werten)
        const vFrame = (time * 60) / 1000;

        const leds = Array(NUM_LEDS).fill({r:0, g:0, b:0});
        const cVal = getLedConfig();
        const baseCol = hexToRgb(cVal.color);
        const { brightness, speed, size, tail, density, hue, hue_shift, saturation, spread } = cVal;

        switch(cVal.effect) {
            case 'Solid':
                leds.fill(applyBrightness(baseCol.r, baseCol.g, baseCol.b, brightness)); break;
            case 'Blink':
                if (Math.floor(vFrame / blink_period_frames(speed)) % 2 === 0)
                    leds.fill(applyBrightness(baseCol.r, baseCol.g, baseCol.b, brightness));
                break;
            case 'Rainbow':
                let base_hue = (vFrame * speed_step(speed)) % 256;
                for (let i = 0; i < NUM_LEDS; i++) {
                    let col = hsv_to_rgb((base_hue - Math.floor((i * 256) / NUM_LEDS) + 256) % 256, 255, 255);
                    leds[i] = applyBrightness(col.r, col.g, col.b, brightness);
                }
                break;
            case 'Breathing':
                let bphase = Math.floor(vFrame * speed_step(speed) * 2) % 512;
                let bval = bphase < 256 ? bphase : 511 - bphase;
                leds.fill(applyBrightness(baseCol.r, baseCol.g, baseCol.b, Math.floor((brightness * bval) / 255)));
                break;
            case 'Chase':
                let chead = Math.floor(vFrame / chase_period_frames(speed)) % NUM_LEDS;
                for (let offset = 0; offset < Math.max(1, size); offset++)
                    leds[(chead + offset) % NUM_LEDS] = applyBrightness(baseCol.r, baseCol.g, baseCol.b, brightness);
                break;
            case 'Comet':
                let cohead = Math.floor(vFrame / comet_period_frames(speed)) % NUM_LEDS;
                let ctail = Math.max(1, tail);
                for (let i = 0; i <= ctail; i++) {
                    let idx = (cohead - i + NUM_LEDS) % NUM_LEDS;
                    let fade = Math.max(0, 255 - Math.floor((i * 255) / (ctail + 1)));
                    leds[idx] = applyBrightness(baseCol.r, baseCol.g, baseCol.b, scale(brightness, fade));
                }
                break;
            case 'Sparkle':
                let sparks = 1 + Math.floor((density * (NUM_LEDS - 1)) / 255);
                for (let i = 0; i < sparks; i++)
                    if(Math.random() > 0.95) leds[Math.floor(Math.random() * NUM_LEDS)] = applyBrightness(baseCol.r, baseCol.g, baseCol.b, brightness);
                break;
            case 'ColorOrbit':
                let orot = Math.floor(vFrame / orbit_period_frames(speed)) % 256;
                for (let i = 0; i < NUM_LEDS; i++) {
                    let offset = Math.floor((i * 256) / NUM_LEDS);
                    let cur_hue = (hue + scale(hue_shift, smooth_wave8((orot - offset + 256) % 256))) % 256;
                    let col = hsv_to_rgb(cur_hue, saturation, 255);
                    leds[i] = applyBrightness(col.r, col.g, col.b, brightness);
                }
                break;
            case 'Astolfo':
                const period = orbit_period_frames(speed);
                const rot = Math.floor((vFrame * 3) / period) % 256;
                const phase_span = 64 + Math.floor((spread * 128) / 255);

                for (let i = 0; i < NUM_LEDS; i++) {
                    const offset = Math.floor((i * phase_span) / NUM_LEDS);
                    const phase = (rot - offset + 256) % 256;
                    const mix = smooth_wave8(phase);
                    const ahue = lerp8(236, 150, mix);
                    const pulse = smooth_wave8((phase + rot) % 256);
                    const aval = Math.min(255, 90 + scale(165, pulse));
                    const raw = hsv_to_rgb(ahue, saturation, aval);
                    leds[i] = applyBrightness(raw.r, raw.g, raw.b, brightness);
                }
                break;
        }

        const c = leds.map(l => `rgb(${l.r}, ${l.g}, ${l.b})`);
        // Mapping: Links -> Unten -> Rechts (Flussrichtung Korrektur)
        leftGrad.value = `linear-gradient(to bottom, ${c[0]}, ${c[1]}, ${c[2]}, ${c[3]})`;
        bottomGrad.value = `linear-gradient(to right, ${c[3]}, ${c[4]}, ${c[5]}, ${c[6]}, ${c[7]}, ${c[8]})`;
        rightGrad.value = `linear-gradient(to top, ${c[8]}, ${c[9]}, ${c[10]}, ${c[11]}, ${c[12]})`;

        rafId = requestAnimationFrame(renderLoop);
    };

    onMounted(() => { rafId = requestAnimationFrame(renderLoop); });
    onUnmounted(() => { if (rafId !== null) cancelAnimationFrame(rafId); });

    return { leftGrad, bottomGrad, rightGrad };
}