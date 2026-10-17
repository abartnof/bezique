// What the tutorial remembers between sittings: the keys of the pages shown,
// in the browser's storage. Every access survives storage that is missing,
// full or blocked (the pages are then remembered for the sitting only).
// The storage is injected, so a test never writes the real key.

export const KEY = "bezique.tutor.v1";

export function createTutorStore(storage) {
  let seen = [];
  try {
    const kept = JSON.parse(storage?.getItem(KEY) ?? "null");
    if (kept && kept.v === 1 && Array.isArray(kept.seen)) seen = kept.seen.filter((k) => typeof k === "string");
  } catch {
    seen = [];
  }
  const save = () => {
    try {
      storage?.setItem(KEY, JSON.stringify({ v: 1, seen }));
    } catch {
      /* the pages are remembered for this sitting only */
    }
  };
  return {
    seen: () => [...seen],
    add(key) {
      if (seen.includes(key)) return;
      seen.push(key);
      save();
    },
    // "Reset the tutorial": every page falls due again.
    reset() {
      seen = [];
      try {
        storage?.removeItem(KEY);
      } catch {
        /* nothing kept to forget */
      }
    },
  };
}
