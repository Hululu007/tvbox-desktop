import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import PosterCard from "../components/PosterCard";
import { api } from "../api";
import { useApp } from "../store";
import type { HomeSectionData } from "../types";

export default function Home() {
  const nav = useNavigate();
  const { state, history } = useApp();
  const [sections, setSections] = useState<HomeSectionData[]>([]);
  const [loading, setLoading] = useState(true);
  const [kw, setKw] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setSections(await api.getHomeSections());
    } catch (e) {
      useApp.getState().toast(String(e), "error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load, state?.activeId]);

  const continueList = history.slice(0, 8);

  return (
    <div>
      <div className="row" style={{ marginBottom: 24 }}>
        <div>
          <h1 className="page-title">首页</h1>
          <p className="page-desc" style={{ marginBottom: 0 }}>
            来自当前订阅的推荐内容
          </p>
        </div>
        <div className="spacer" />
        <input
          className="input"
          style={{ width: 280 }}
          placeholder="搜索影视名，回车聚合搜索…"
          value={kw}
          onChange={(e) => setKw(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && kw.trim()) {
              nav(`/search?q=${encodeURIComponent(kw.trim())}`);
            }
          }}
        />
        <button className="btn" onClick={load}>
          刷新推荐
        </button>
      </div>

      {continueList.length > 0 && (
        <div className="section">
          <div className="section-head">
            <h3>继续观看</h3>
            <span className="from">观看进度已自动保存</span>
          </div>
          <div className="hscroll">
            {continueList.map((h) => (
              <PosterCard
                key={`${h.site_key}-${h.vod_id}`}
                card={{
                  site_key: h.site_key,
                  site_name: h.site_name,
                  vod_id: h.vod_id,
                  name: h.name,
                  pic: h.pic,
                  remarks: h.remarks,
                }}
                progress={h}
              />
            ))}
          </div>
        </div>
      )}

      {loading ? (
        <div className="loading">正在加载推荐位…</div>
      ) : sections.length === 0 ? (
        <div className="empty">
          当前订阅没有可展示的内容。
          <br />
          请到「设置」添加或刷新订阅，或到「影视源」直接浏览站点。
        </div>
      ) : (
        sections.map((sec) => (
          <div className="section" key={sec.title + sec.site_key}>
            <div className="section-head">
              <h3>{sec.title}</h3>
              <span className="from">来自 {sec.site_name}</span>
            </div>
            <div className="hscroll">
              {sec.items.map((c) => (
                <PosterCard key={`${c.site_key}-${c.vod_id}`} card={c} />
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  );
}
