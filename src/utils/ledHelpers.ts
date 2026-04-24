/**
 * Konvertiert einen Hex-String (#RRGGBB) in ein RGB-Objekt.
 */
export const hexToRgb = (hex: string) => {
    // Entfernt das '#' falls vorhanden
    const cleanHex = hex.startsWith('#') ? hex.slice(1) : hex;

    const r = parseInt(cleanHex.slice(0, 2), 16);
    const g = parseInt(cleanHex.slice(2, 4), 16);
    const b = parseInt(cleanHex.slice(4, 6), 16);

    return { r, g, b };
};