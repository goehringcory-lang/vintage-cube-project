export const PULSE_RING = {
  strokeWidth: 3,
  periodS: 1,
  minAlpha: 0.45,
  halo: [
    { offset: 3, width: 3, alpha: 0.55 },
    { offset: 7, width: 5, alpha: 0.3 },
    { offset: 12, width: 7, alpha: 0.14 },
  ],
} as const;
