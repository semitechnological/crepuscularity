import { useSyncExternalStore } from "react";

type Listener = () => void;

type Trackable = {
  subscribe: (listener: Listener) => () => void;
};

let batchDepth = 0;
const pending = new Set<Listener>();
let currentTracker: ((dep: Trackable) => void) | null = null;

/** Coalesce nested writes into one subscriber flush. */
export function batch(fn: () => void): void {
  batchDepth++;
  try {
    fn();
  } finally {
    batchDepth--;
    if (batchDepth === 0 && pending.size > 0) {
      const queue = [...pending];
      pending.clear();
      for (const listener of queue) listener();
    }
  }
}

function notifyListeners(listeners: Set<Listener>): void {
  if (batchDepth > 0) {
    for (const listener of listeners) pending.add(listener);
    return;
  }
  for (const listener of [...listeners]) listener();
}

function track(dep: Trackable): void {
  currentTracker?.(dep);
}

export type Signal<T> = {
  (): T;
  set: (value: T | ((prev: T) => T)) => void;
  peek: () => T;
  subscribe: (listener: Listener) => () => void;
};

/**
 * Solid-inspired signal: read by calling, write via `.set`.
 * React components subscribe with `useSignal`.
 */
export function createSignal<T>(initial: T): Signal<T> {
  let value = initial;
  const listeners = new Set<Listener>();
  const node: Trackable = {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };

  const read = (() => {
    track(node);
    return value;
  }) as Signal<T>;

  read.set = (next) => {
    const resolved = typeof next === "function" ? (next as (prev: T) => T)(value) : next;
    if (Object.is(resolved, value)) return;
    value = resolved;
    notifyListeners(listeners);
  };
  read.peek = () => value;
  read.subscribe = node.subscribe;

  return read;
}

export type Memo<T> = {
  (): T;
  peek: () => T;
  subscribe: (listener: Listener) => () => void;
};

/**
 * Derived value that recomputes when tracked dependencies change.
 * Any `createSignal` / `createMemo` read during `compute` is tracked.
 */
export function createMemo<T>(compute: () => T): Memo<T> {
  const listeners = new Set<Listener>();
  const unsubs: Array<() => void> = [];
  let value!: T;
  let stale = true;

  const node: Trackable = {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };

  const recompute = () => {
    for (const unsub of unsubs.splice(0)) unsub();

    const previous = currentTracker;
    const seen = new Set<Trackable>();
    currentTracker = (dep) => {
      if (seen.has(dep)) return;
      seen.add(dep);
      unsubs.push(
        dep.subscribe(() => {
          stale = true;
          const next = peekFresh();
          notifyListeners(listeners);
          void next;
        }),
      );
    };

    try {
      value = compute();
      stale = false;
    } finally {
      currentTracker = previous;
    }
  };

  const peekFresh = (): T => {
    if (stale) recompute();
    return value;
  };

  // Seed dependencies
  recompute();

  const read = (() => {
    track(node);
    return peekFresh();
  }) as Memo<T>;

  read.peek = peekFresh;
  read.subscribe = node.subscribe;

  return read;
}

export type StoreSetter<T extends object> = (fn: (state: T) => void) => void;

const storeRoots = new WeakMap<object, Trackable>();

/**
 * Nested reactive store (Solid-inspired).
 * Mutate through the proxy (or `setStore`) to notify subscribers.
 */
export function createStore<T extends object>(initial: T): [T, StoreSetter<T>] {
  const listeners = new Set<Listener>();
  const state = structuredClone(initial) as T;

  const node: Trackable = {
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };

  const notify = () => notifyListeners(listeners);

  const wrap = <V extends object>(target: V): V =>
    new Proxy(target, {
      get(obj, prop, receiver) {
        track(node);
        const value = Reflect.get(obj, prop, receiver);
        if (value !== null && typeof value === "object") {
          return wrap(value as object);
        }
        return value;
      },
      set(obj, prop, value, receiver) {
        const prev = Reflect.get(obj, prop, receiver);
        if (Object.is(prev, value)) return true;
        const ok = Reflect.set(obj, prop, value, receiver);
        if (ok) notify();
        return ok;
      },
      deleteProperty(obj, prop) {
        if (!Object.prototype.hasOwnProperty.call(obj, prop)) return true;
        const ok = Reflect.deleteProperty(obj, prop);
        if (ok) notify();
        return ok;
      },
    });

  const proxy = wrap(state);
  storeRoots.set(proxy, node);

  const setStore: StoreSetter<T> = (fn) => {
    batch(() => {
      fn(proxy);
    });
  };

  return [proxy, setStore];
}

/** React hook: re-render when a signal or memo changes. */
export function useSignal<T>(source: {
  (): T;
  subscribe: (listener: Listener) => () => void;
}): T {
  const subscribe = (onStoreChange: () => void) => source.subscribe(onStoreChange);
  const getSnapshot = () => source();
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

/** React hook: re-render when a `createStore` proxy mutates. */
export function useStore<T extends object>(store: T): T {
  const node = storeRoots.get(store as object);
  if (!node) {
    throw new Error("useStore: expected a createStore() proxy");
  }
  useSyncExternalStore(node.subscribe, () => store, () => store);
  return store;
}
