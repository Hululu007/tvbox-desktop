# 架构与数据流

## 总体结构

```
┌────────────────────────── Tauri 主窗口（main，React） ──────────────────────────┐
│  pages: Home / Sources / Search / Detail / Live / Favorites / History / Settings │
│  store(zustand) ── api.ts(invoke 封装) ── HashRouter                              │
│  Player: ArtPlayer + hls.js（m3u8 / 直链）                                        │
└───────────────┬──────────────────────────────────────────────────┬──────────────┘
                │ invoke("command", args)                           │ listen("event")
┌───────────────▼──────────────────── Tauri 2 Core ────────────────▼──────────────┐
│  commands.rs（25 个命令，薄封装）                                                  │
│    ├─ 订阅: get_app_state / add_subscription_{url,local,raw} /                   │
│    │        remove / set_active / refresh                                        │
│    ├─ 内容: get_home_sections / get_categories / get_videolist / get_detail      │
│    ├─ 搜索: aggregate_search（并发调度）                                          │
│    ├─ 直播: get_live_channels / speed_test_lines / resolve_room_url              │
│    ├─ 外部播放器: open_external_player（自动探测 + 指定路径）                        │
│    ├─ 收藏历史: add/remove/get_favorites / save_progress / get/remove/clear      │
│    └─ 嗅探: sniff_page（HTML直取）/ open_sniffer / close_sniffer                  │
│                                                                                  │
│  state.rs  AppState { data_dir, reqwest::Client, Mutex<RuntimeState> }           │
│  storage.rs JSON 文件: subscriptions.json / favorites.json / history.json        │
│  cms.rs / htmlsite.rs / search.rs / live.rs / room.rs / sniffer.rs / external.rs  │
└──────────────────────────────────────────────────────────────────────────────────┘
```

## 关键数据流

### 1. 订阅生命周期
```
添加(URL/本地/粘贴) → fetch_text(GBK兜底) → TvboxConfig::parse_json
  → sites[].normalize()（type 归一化 kind + unsupported_reason）
  → 存 RuntimeState + subscriptions.json → 前端 reloadState
```
激活订阅只切换 `active_id`；所有内容命令都从 `active_config()` 取站点。

### 2. 分类浏览
```
get_categories(siteKey)   → cms.rs: ?ac=list（class 为空时回退 videolist）
get_videolist(siteKey,t,p)→ cms.rs: ?ac=videolist&t=&pg= → PagedVideos
```
URL 构造支持两种 api 写法：基地址直接拼 query，或 `{ac}{t}{pg}{wd}{ids}` 占位符模板。

### 3. 聚合搜索
```
aggregate_search(wd)
  → 过滤 is_searchable()（cms/api/html 且 searchable=1 且 enabled≠0）
  → futures stream buffer_unordered(6)，单站 12s 超时
  → 每完成 emit "search:progress" {done,total,siteKey,ok,count}
  → 全部完成返回 Vec<SearchResultGroup>
```

### 4. 播放与进度记忆
```
选集点击 → 三分支：
  a) room://（huya/douyin）→ resolve_room_url → 真实 m3u8
  b) is_web（非直链网页）  → sniff_page（HTML直取）
                            └ 空结果 → open_sniffer（webview hook 窗口）
                            └ sniff:found 事件 → foundLinks 选择
  c) 直链                  → setPlayUrl
Player(ArtPlayer+hls.js) → timeupdate 节流 8s → save_progress(upsert, 截断200条)
详情页打开 → 匹配 history → 自动定位 source/episode + startPos
```

### 5. 直播
```
get_live_channels → 遍历 config.lives
  → B格式 {name,url}: fetch_text → parse_live_text(txt/m3u)
  → A格式 {groups[]}: 直接展开
  → 同组同名频道 lines 合并
speed_test_lines(urls) → 并发6路 GET Range 0-2048 / 6s 超时 → emit "live:speed"
```

## 序列化约定

- Rust → 前端字段保持 snake_case（`site_key`、`vod_id`…），前端 TS 类型一一对应。
- 前端 → Rust 参数用 camelCase（`siteKey`、`vodId`…），Tauri 2 自动映射到 snake_case。
- 事件名：`search:progress`、`sniff:found`、`live:speed`。

## 存储位置（app_data_dir）

| 平台 | 路径 |
| --- | --- |
| Windows | `%APPDATA%\com.tvbox.desktop` |
| macOS | `~/Library/Application Support/com.tvbox.desktop` |
| Linux | `~/.config/com.tvbox.desktop` |

首次启动若无 `subscriptions.json`，自动写入内置订阅（builtin/public-sources.json，来自原项目 samples，MIT）。

## 并发与超时参数

| 场景 | 参数 |
| --- | --- |
| 全局 HTTP 超时 | 25s |
| CMS 请求超时 | 15s（站点可覆盖 timeout） |
| 聚合搜索单站 | 12s，并发 6 |
| 直播测速单线路 | 6s，并发 6 |
| 首页 section | 12s，全并发 |
