// 全局状态：应用状态（订阅）+ 应用设置 + 轻提示
import { create } from "zustand";
import { api } from "./api";
import type { AppStateDto, AppSettings, FavoriteItem, HistoryItem } from "./types";

interface Toast {
  id: number;
  text: string;
  kind: "info" | "error" | "success";
}

interface AppStore {
  state: AppStateDto | null;
  stateLoading: boolean;
  settings: AppSettings | null;
  favorites: FavoriteItem[];
  history: HistoryItem[];
  toasts: Toast[];
  toast: (text: string, kind?: Toast["kind"]) => void;
  reloadState: () => Promise<void>;
  reloadSettings: () => Promise<void>;
  applyAppearance: (s: AppSettings) => void;
  reloadFavorites: () => Promise<void>;
  reloadHistory: () => Promise<void>;
  reloadAll: () => Promise<void>;
}

let toastSeq = 0;

// 强调色 -> 浅背景色（透明度 12%）
function accentSoft(hex: string): string {
  const m = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m) return "#eef2ff";
  const n = parseInt(m[1], 16);
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, 0.12)`;
}

export const useApp = create<AppStore>((set, get) => ({
  state: null,
  stateLoading: true,
  settings: null,
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

  reloadSettings: async () => {
    try {
      const s = await api.getSettings();
      set({ settings: s });
      get().applyAppearance(s);
    } catch {
      /* 忽略 */
    }
  },

  // 强调色与界面密度写入 CSS，全局生效
  applyAppearance: (s: AppSettings) => {
    const root = document.documentElement;
    root.style.setProperty("--accent", s.accent);
    root.style.setProperty("--accent-soft", accentSoft(s.accent));
    document.body.classList.toggle("compact", s.density === "compact");
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
      get().reloadSettings(),
      get().reloadFavorites(),
      get().reloadHistory(),
    ]);
  },
}));
