import type { UiConfig } from './ui.types';

// Explicit props override component defaults, then global defaults; components own fallbacks.
const uiConfig: UiConfig = {} satisfies UiConfig;

export default uiConfig;
