import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DisguiseState } from "../types";

const DEFAULT_STATE: DisguiseState = {
  supported: false,
  currentName: "insomniAPP",
  isDisguised: false,
};

export function useDisguiseState() {
  const [state, setState] = useState<DisguiseState>(DEFAULT_STATE);

  useEffect(() => {
    invoke<DisguiseState>("get_disguise_state")
      .then(setState)
      .catch(() => {
        setState(DEFAULT_STATE);
      });
  }, []);

  return state;
}
