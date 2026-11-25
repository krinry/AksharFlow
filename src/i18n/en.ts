export interface Dictionary {
  appName: string;
  tagline: string;
  layouts: {
    en_qwerty: string;
    hi_inscript: string;
    hi_remington_gail: string;
  };
  metrics: {
    wpm: string;
    netWpm: string;
    grossWpm: string;
    accuracy: string;
    consistency: string;
    errors: string;
    line: string;
    time: string;
  };
  typing: {
    focusPrompt: string;
    capsLockOn: string;
    capsLockHindiHint: string;
    pendingShortI: string;
    restart: string;
    restartShortcut: string;
    zenMode: string;
    exitZen: string;
    touchModeHint: string;
    nextKey: string;
    withShift: string;
    withAltGr: string;
  };
  adaptive: {
    unlockedKeys: string;
    weakFocus: string;
    keyUnlockedTitle: string;
    confidenceLabel: string;
    heatmap: string;
  };
  settings: {
    title: string;
    close: string;
    targetWpm: string;
    targetAccuracy: string;
    unlockThreshold: string;
    volume: string;
    backupTitle: string;
    exportJson: string;
    importMerge: string;
    importReplace: string;
    resetLayout: string;
    confirmReset: string;
    exportSuccess: string;
    importSuccess: string;
    resetSuccess: string;
  };
  sound: {
    label: string;
    off: string;
    blue: string;
    brown: string;
    red: string;
    topre: string;
  };
  fingers: {
    left_pinky: string;
    left_ring: string;
    left_middle: string;
    left_index: string;
    thumb: string;
    right_index: string;
    right_middle: string;
    right_ring: string;
    right_pinky: string;
  };
  theme: {
    light: string;
    dark: string;
  };
}

export const en: Dictionary = {
  appName: "AksharFlow",
  tagline: "Offline-First Adaptive Typing Studio",
  layouts: {
    en_qwerty: "English · QWERTY",
    hi_inscript: "Hindi · InScript",
    hi_remington_gail: "Hindi · Remington GAIL",
  },
  metrics: {
    wpm: "WPM",
    netWpm: "Net WPM",
    grossWpm: "Gross",
    accuracy: "Accuracy",
    consistency: "Consistency",
    errors: "Errors",
    line: "Line",
    time: "Time",
  },
  typing: {
    focusPrompt: "Click here or start typing to begin",
    capsLockOn: "CapsLock ON",
    capsLockHindiHint: "Ignored in Hindi layouts",
    pendingShortI: "Pending ि",
    restart: "New Line",
    restartShortcut: "Tab + Enter",
    zenMode: "Zen Focus",
    exitZen: "Exit Zen",
    touchModeHint: "Tap any key below to type",
    nextKey: "Next",
    withShift: "Shift",
    withAltGr: "AltGr",
  },
  adaptive: {
    unlockedKeys: "Unlocked",
    weakFocus: "Focus",
    keyUnlockedTitle: "New Key Unlocked",
    confidenceLabel: "Confidence",
    heatmap: "Heatmap",
  },
  settings: {
    title: "Adaptive Engine & Backup",
    close: "Close",
    targetWpm: "Target Speed (WPM)",
    targetAccuracy: "Min Accuracy (%)",
    unlockThreshold: "Unlock Confidence (%)",
    volume: "Audio Volume",
    backupTitle: "Local Backup & Progress (.json)",
    exportJson: "Export Progress (.json)",
    importMerge: "Import & Merge",
    importReplace: "Import & Replace",
    resetLayout: "Reset Current Layout",
    confirmReset: "Click again to confirm reset",
    exportSuccess: "Backup exported successfully.",
    importSuccess: "Backup restored into local database.",
    resetSuccess: "Layout progress reset to Home Row.",
  },
  sound: {
    label: "Switch Audio",
    off: "Muted",
    blue: "Clicky Blue",
    brown: "Tactile Brown",
    red: "Linear Red",
    topre: "Topre Thock",
  },
  fingers: {
    left_pinky: "Left Pinky",
    left_ring: "Left Ring",
    left_middle: "Left Middle",
    left_index: "Left Index",
    thumb: "Thumb",
    right_index: "Right Index",
    right_middle: "Right Middle",
    right_ring: "Right Ring",
    right_pinky: "Right Pinky",
  },
  theme: {
    light: "Light",
    dark: "Dark",
  },
};
