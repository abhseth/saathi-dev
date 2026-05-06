import React from "react";

const TIMETABLE_KEY = "saathi:offline:timetable";
const SUBSTITUTIONS_KEY = "saathi:offline:substitutions";
const SYNC_KEY = "saathi:offline:needsSync";

export function useOfflineCache() {
  const [isOnline, setIsOnline] = React.useState<boolean>(navigator.onLine);
  const [needsSync, setNeedsSync] = React.useState<boolean>(() => {
    try {
      return localStorage.getItem(SYNC_KEY) === "1";
    } catch {
      return false;
    }
  });

  React.useEffect(() => {
    const handleOnline = () => {
      setIsOnline(true);
      try {
        if (localStorage.getItem(SYNC_KEY) === "1") {
          setNeedsSync(true);
        }
      } catch (e) {
        console.warn("Offline cache read failed:", e);
      }
    };
    const handleOffline = () => setIsOnline(false);
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);
    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  }, []);

  const cacheTimetable = React.useCallback((data: unknown) => {
    try {
      localStorage.setItem(TIMETABLE_KEY, JSON.stringify(data));
    } catch (e) {
      console.warn("Offline cache write failed:", e);
    }
  }, []);

  const getCachedTimetable = React.useCallback(() => {
    try {
      const raw = localStorage.getItem(TIMETABLE_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch {
      return null;
    }
  }, []);

  const cacheSubstitutions = React.useCallback((data: unknown) => {
    try {
      localStorage.setItem(SUBSTITUTIONS_KEY, JSON.stringify(data));
    } catch (e) {
      console.warn("Offline cache write failed:", e);
    }
  }, []);

  const getCachedSubstitutions = React.useCallback(() => {
    try {
      const raw = localStorage.getItem(SUBSTITUTIONS_KEY);
      return raw ? JSON.parse(raw) : null;
    } catch {
      return null;
    }
  }, []);

  const flagNeedsSync = React.useCallback(() => {
    try {
      localStorage.setItem(SYNC_KEY, "1");
      setNeedsSync(true);
    } catch (e) {
      console.warn("Offline cache write failed:", e);
    }
  }, []);

  const clearSyncFlag = React.useCallback(() => {
    try {
      localStorage.removeItem(SYNC_KEY);
      setNeedsSync(false);
    } catch (e) {
      console.warn("Offline cache write failed:", e);
    }
  }, []);

  return {
    isOnline,
    needsSync,
    cacheTimetable,
    getCachedTimetable,
    cacheSubstitutions,
    getCachedSubstitutions,
    flagNeedsSync,
    clearSyncFlag,
  };
}
