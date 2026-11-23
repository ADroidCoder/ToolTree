import QRCode, { type QRCodeErrorCorrectionLevel, type QRCodeRenderersOptions } from "qrcode";

export type QrLevel = "L" | "M" | "Q" | "H";

export interface QrConfig {
  level: QrLevel;
  size: number;
  margin: number;
  dark: string;
  light: string;
}

function clamp(value: number, min: number, max: number, fallback: number) {
  return Number.isFinite(value) ? Math.max(min, Math.min(max, Math.round(value))) : fallback;
}

export function qrOptions(config: QrConfig): QRCodeRenderersOptions {
  return {
    errorCorrectionLevel: config.level as QRCodeErrorCorrectionLevel,
    width: clamp(Number(config.size), 128, 2048, 512),
    margin: clamp(Number(config.margin), 0, 10, 4),
    color: { dark: `${config.dark}ff`, light: `${config.light}ff` },
  };
}

export function qrVersion(text: string, level: QrLevel) {
  return QRCode.create(text, { errorCorrectionLevel: level }).version;
}
