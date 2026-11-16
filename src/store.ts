// 全局状态：应用状态（订阅）+ 轻提示
import { create } from "zustand";
import { api } from "./api";
import type { AppStateDto, FavoriteItem, HistoryItem } from "./types";

interface Toast {
  id: number;
  text: string;
  kind: "info" | "error" | "success";
}

interface AppStore {
  state: AppStateDto | null;
  stateLoading: boolean;
  favorites: FavoriteItem[];
  history: HistoryItem[];
  toasts: Toast[];
  toast: (text: string, kind?: Toast["kind"]) => void;
  reloadState: () => Promise<void>;
  reloadFavorites: () => Promise<void>;
  reloadHistory: () => Promise<void>;
  reloadAll: () => Promise<void>;
}

let toastSeq = 0;

export const useApp = create<AppStore>((set, get) => ({
  state: null,
  stateLoading: true,
  favorites: [],
  history: [],
  toasts: [],

  toast: (text, kind = "info") => {
    const id = ++toastSeq;
    set((s) => ({ toasts: [...s.toasts, { id, text, kind }] }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) }));
    }, kind === "error" ? 6000 : 3200);
  },

  reloadState: async () => {
    try {
      const st = await api.getAppState();
      set({ state: st, stateLoading: false });
    } catch (e) {
      get().toast(String(e), "error");
      set({ stateLoading: false });
    }
  },

  reloadFavorites: async () => {
    try {
      set({ favorites: await api.getFavorites() });
    } catch {
      /* 忽略 */
    }
  },

  reloadHistory: async () => {
    try {
      set({ history: await api.getHistory() });
    } catch {
      /* 忽略 */
    }
  },

  reloadAll: async () => {
    await Promise.all([
      get().reloadState(),
      get().reloadFavorites(),
      get().reloadHistory(),
    ]);
  },
}));
