export interface AppStatus {
  enabled: boolean;
  isIdle: boolean;
  idleSeconds: number;
  isInMeeting: boolean;
  isSessionLocked: boolean;
  isDisplayOff: boolean;
  isSimulating: boolean;
  idleThresholdSecs: number;
  simulationIntervalSecs: number;
}

export interface SettingsPayload {
  idleThresholdSecs?: number;
  simulationIntervalSecs?: number;
}

export interface DisguiseState {
  supported: boolean;
  currentName: string;
  isDisguised: boolean;
}
