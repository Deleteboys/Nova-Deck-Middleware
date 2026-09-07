import {computed, ref} from 'vue';
import {getPlatform, type HostPlatform} from '@/services/streamdeckCommands';

// Die Plattform ändert sich zur Laufzeit nicht, deshalb wird sie einmal pro
// App-Instanz geladen und von allen Aufrufern geteilt.
const platform = ref<HostPlatform>('unknown');
let pending: Promise<HostPlatform> | null = null;

export function usePlatform() {
    if (!pending) {
        pending = getPlatform()
            .then((value) => (platform.value = value))
            .catch((error) => {
                console.error('Plattform konnte nicht ermittelt werden:', error);
                return platform.value;
            });
    }

    return {
        platform,
        isWindows: computed(() => platform.value === 'windows'),
        isLinux: computed(() => platform.value === 'linux'),
    };
}
