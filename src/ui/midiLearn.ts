/**
 * MIDI controller knobs (control change messages) for the encoders. Any encoder can be bound to a
 * CC number by "learning" it: turn learn on, click an encoder, move a knob on the controller. The
 * encoder then follows that knob over its whole range (CC 0..127 → min..max, logarithmic where
 * the encoder is). Bindings are kept on the device, keyed by the encoder group and its label.
 * Every encoder of a group listens, whichever page of it is showing.
 */
import { useSyncExternalStore } from "react";

interface Spec {
  min: number;
  max: number;
  log?: boolean;
  onChange: (v: number) => void;
}

const KEPT = "chimera-midi-cc";

interface State {
  bindings: Readonly<Record<string, number>>;
  /** learn mode: clicking an encoder arms it instead of turning it */
  learn: boolean;
  /** the encoder waiting for a CC */
  armed: string | null;
}

function kept(): Record<string, number> {
  try {
    const k = JSON.parse(localStorage.getItem(KEPT) ?? "null") as Record<string, unknown> | null;
    const out: Record<string, number> = {};
    for (const [id, cc] of Object.entries(k ?? {})) if (typeof cc === "number" && cc >= 0 && cc < 128) out[id] = cc;
    return out;
  } catch {
    return {}; // no storage here: start fresh
  }
}

let state: State = { bindings: kept(), learn: false, armed: null };
const subs = new Set<() => void>();
const registry = new Map<string, Spec>();

function set(next: Partial<State>): void {
  const bindingsChanged = next.bindings !== undefined;
  state = { ...state, ...next };
  if (bindingsChanged) {
    try {
      localStorage.setItem(KEPT, JSON.stringify(state.bindings));
    } catch {
      // no storage: the bindings last for this visit
    }
  }
  subs.forEach((f) => f());
}

export function useMidiLearn(): State {
  return useSyncExternalStore(
    (f) => {
      subs.add(f);
      return () => subs.delete(f);
    },
    () => state,
  );
}

export const encoderId = (group: string, label: string) => `${group}/${label}`;

/** An encoder listens while registered; returns how to stop. */
export function registerEncoder(id: string, spec: Spec): () => void {
  registry.set(id, spec);
  return () => {
    if (registry.get(id) === spec) registry.delete(id);
  };
}

export function setLearn(on: boolean): void {
  if (state.learn === on && state.armed === null) return;
  set({ learn: on, armed: null });
}

/** Click on an encoder in learn mode: arm it; again: clear its CC and disarm. */
export function pick(id: string): void {
  if (state.armed === id) {
    const rest = { ...state.bindings };
    delete rest[id];
    set({ bindings: rest, armed: null });
  } else {
    set({ armed: id });
  }
}

/** The value `u` (0..1) of the way along an encoder's range. */
export function valueAt(s: Pick<Spec, "min" | "max" | "log">, u: number): number {
  const c = Math.min(1, Math.max(0, u));
  return s.log ? Math.exp(Math.log(s.min) + c * (Math.log(s.max) - Math.log(s.min))) : s.min + c * (s.max - s.min);
}

/** A control change from the controller: binds the armed encoder, else moves the bound ones. */
export function handleCc(cc: number, value: number): void {
  if (state.armed) {
    const bindings: Record<string, number> = {};
    // one knob drives one encoder: a CC taken from another gives way
    for (const [id, c] of Object.entries(state.bindings)) if (c !== cc) bindings[id] = c;
    bindings[state.armed] = cc;
    set({ bindings, armed: null });
    return;
  }
  for (const [id, c] of Object.entries(state.bindings)) {
    if (c !== cc) continue;
    const s = registry.get(id);
    if (s) s.onChange(valueAt(s, value / 127));
  }
}
