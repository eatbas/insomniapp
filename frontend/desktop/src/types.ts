/**
 * How the backend resets the operating system's input idle counter while the
 * session is idle. `mouseNudge` is a zero-delta pointer move that no
 * application can observe; `f15` is a real keypress, kept only as a fallback
 * for input stacks that discard zero-delta moves.
 */
export type NudgeMethod = "mouseNudge" | "f15";

export interface AppStatus {
  enabled: boolean;
  isIdle: boolean;
  idleSeconds: number;
  isSessionLocked: boolean;
  isDisplayOff: boolean;
  isSimulating: boolean;
  idleThresholdSecs: number;
  simulationIntervalSecs: number;
  nudgeMethod: NudgeMethod;
}

export interface SettingsPayload {
  idleThresholdSecs?: number;
  simulationIntervalSecs?: number;
  nudgeMethod?: NudgeMethod;
}

export interface DisguiseState {
  supported: boolean;
  currentName: string;
  isDisguised: boolean;
}
