export interface AppStatus {
  enabled: boolean;
  isIdle: boolean;
  idleSeconds: number;
  isInMeeting: boolean;
  isSimulating: boolean;
  idleThresholdSecs: number;
  simulationIntervalSecs: number;
}

export interface SettingsPayload {
  idleThresholdSecs?: number;
  simulationIntervalSecs?: number;
}
