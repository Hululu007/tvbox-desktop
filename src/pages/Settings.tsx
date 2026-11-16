import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { useApp } from "../store";
import { siteKindOk } from "../types";

export default function Settings() {
  const { state, reloadAll, toast } = useApp();
  const [url, setUrl] = useState("");
  const [raw, setRaw] = useState("");
  const [busy, setBusy] = useState("");

  const subs = state?.subscriptions ?? [];
  const activeId = state?.activeId ?? null;
  const activeCfg = state?.activeConfig ?? null;
  const okCount = activeCfg?.sites.filter(siteKindOk).length ?? 0;

  const addUrl = async () => {
    if (!url.trim()) return;
    setBusy("url");
    try {
      const sub = await api.addSubscriptionUrl(url.trim());
      toast(`订阅「${sub.name}」添加成功`, "success");
      setUrl("");
      await reloadAll();
    } catch (e) {
      toast(`添加失败: ${String(e)}`, "error");
    } finally {
      setBusy("");
    }
  };

  const addRaw = async () => {
    if (!raw.trim()) return;
    setBusy("raw");
    try {
      const sub = await api.addSubscriptionRaw(raw);
      toast(`订阅「${sub.name}」添加成功`, "success");
      setRaw("");
      await reloadAll();
    } catch (e) {
      toast(`添加失败: ${String(e)}`, "error");
    } finally {
      setBusy("");
    }
  };

  const addLocal = async () => {
    setBusy("local");
    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "TVBox 配置", extensions: ["json"] }],
      });
      if (!path) return;
      const sub = await api.addSubscriptionLocal(path as string);
      toast(`订阅「${sub.name}」添加成功`, "success");
      await reloadAll();
    } catch (e) {
      toast(`添加失败: ${String(e)}`, "error");
    } finally {
      setBusy("");
    }
  };

  const act = async (fn: () => Promise<unknown>, msg: string) => {
    setBusy("op");
    try {
      await fn();
      await reloadAll();
    } catch (e) {
      toast(`${msg}: ${String(e)}`, "error");
    } finally {
      setBusy("");
    }
  };

  return (
    <div>
      <h1 className="page-title">设置</h1>
      <p className="page-desc">
        订阅管理：支持 TVBox 订阅链接、本地 JSON 文件、粘贴配置三种方式；扩展字段写在
        tvboxDesktop 键下
      </p>

      {/* 添加订阅 */}
      <div className="card" style={{ padding: 18, marginBottom: 22 }}>
        <b style={{ display: "block", marginBottom: 12 }}>添加订阅</b>
        <div className="row" style={{ marginBottom: 12, flexWrap: "wrap" }}>
          <input
            className="input"
            style={{ flex: 1, minWidth: 320 }}
            placeholder="订阅链接，如 https://example.com/config.json"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addUrl()}
          />
          <button className="btn primary" disabled={busy === "url"} onClick={addUrl}>
            {busy === "url" ? "添加中…" : "添加链接"}
          </button>
          <button className="btn" disabled={busy === "local"} onClick={addLocal}>
            {busy === "local" ? "选择中…" : "选择本地 JSON"}
          </button>
        </div>
        <textarea
          className="input"
          placeholder="或粘贴 TVBox 配置 JSON 内容…"
          value={raw}
          onChange={(e) => setRaw(e.target.value)}
        />
        <div className="row" style={{ marginTop: 10 }}>
          <button className="btn" disabled={busy === "raw" || !raw.trim()} onClick={addRaw}>
            {busy === "raw" ? "添加中…" : "添加粘贴的配置"}
          </button>
        </div>
      </div>

      {/* 订阅列表 */}
      <b style={{ display: "block", marginBottom: 12 }}>
        我的订阅（当前激活订阅含 {activeCfg?.sites.length ?? 0} 个站点，
        其中 {okCount} 个桌面端可用）
      </b>
      {subs.map((s) => {
        const isActive = s.id === activeId;
        const okSites = s.config.sites.filter(siteKindOk).length;
        return (
          <div className="card sub-item" key={s.id}>
            <div style={{ minWidth: 0, flex: 1 }}>
              <div className="row">
                <span className="name">{s.name}</span>
                {isActive ? (
                  <span className="badge">使用中</span>
                ) : (
                  <span className="badge gray">未激活</span>
                )}
                <span className="badge gray">
                  {s.config.sites.length} 站点 / {okSites} 可用
                </span>
              </div>
              <div className="url">{s.url}</div>
            </div>
            <button
              className="btn small"
              disabled={isActive || busy === "op"}
              onClick={() =>
                act(() => api.setActiveSubscription(s.id), "激活失败")
              }
            >
              激活
            </button>
            <button
              className="btn small"
              disabled={busy === "op"}
              onClick={() => act(() => api.refreshSubscription(s.id), "刷新失败")}
            >
              刷新
            </button>
            <button
              className="btn small danger"
              disabled={busy === "op"}
              onClick={() => act(() => api.removeSubscription(s.id), "删除失败")}
            >
              删除
            </button>
          </div>
        );
      })}
      {subs.length === 0 && <div className="empty">暂无订阅，请先添加</div>}
    </div>
  );
}
