# 更新日志

## 1.2.1（2026-11-16）

修复一批 Rust 编译错误，本次已通过 `cargo check --all-targets` 全量真实验证（192 个依赖 + 全部自研代码，零错误零警告）：

- 修复：`ai.rs` E0599 —— 代码使用 `.json()` 但 reqwest 未启用 `json` feature（`Cargo.toml` 补齐）
- 修复：`parse.rs` E0282 —— 删除从未使用、类型无法推断的死变量 `headers`
- 修复：`cms.rs` E0308 —— `apply_template(tpl, ...)` 的 `String` 参数改为借用 `&tpl`
- 修复：`cms.rs` E0382 —— 选集解析先计算 `is_web` 再把 `url` move 进结构体
- 清理：`parse.rs` / `commands.rs` 未用导入，`search.rs` 死常量改接设置默认并发数，`models.rs` 驼峰字段 `warningText` 补 `#[allow(non_snake_case)]`
- 版本号统一 1.2.1（此前 CHANGELOG 与 Cargo.toml / tauri.conf.json / package.json 不一致）

## 1.2.0（2026-11-16）

对齐原版设置页（配置源 / 解析接口 / 通用 / AI 总结 / 关于）：

- 新增：订阅启用开关、重命名、同步时间显示，列表统计（站点/直播/解析数）；禁用订阅自动回退生效配置
- 新增：解析接口（parses）兜底——网页线路嗅探失败后按订阅 parses 顺序依次解析，设置页可测试接口
- 新增：通用设置并持久化到 settings.json：聚合搜索并发数（1-32）、请求超时、界面密度（舒适/紧凑）、强调色（CSS 变量实时生效）、搜索结果过滤解说/速看/混剪、自定义 User-Agent（贯通 CMS/搜索/直播/嗅探全链路）
- 新增：外部播放器命令模板（{url} 占位，支持 --fullscreen 等参数）+「检测本机播放器」
- 新增：AI 总结本集（DeepSeek / OpenAI 兼容接口，Rust 端调用，仅发送片名/集名/类型/年份/演员/简介，不发送播放地址）
- 新增：关于区（版本 / 数据目录一键打开 / 运行环境 / 免责声明）
- Tauri 命令 25 → 33 个，新增模块 settings.rs / parse.rs / ai.rs

## 1.0.1（2026-11-16）

- 新增：一键外部播放器。详情页/直播页/嗅探结果均可把直链交给本机 PotPlayer / VLC / mpv / IINA；未配置时按常见安装路径自动探测，设置页可手动指定程序路径（Rust `std::process` 拉起，新增 `open_external_player` 命令与 `external.rs` 模块）
- 修复：Windows 下 `tauri dev` 启动即崩（Vite 监听 `src-tauri/target/` 碰到 Cargo 锁定的 .exe 抛 EBUSY，现已在 `vite.config.ts` 忽略 Rust 构建目录）
- 修复：`cms.rs` 两处编译错误（模板参数 `&str` 借用、选集解析 move 后借用）

## 1.0.0（2026-11-16）

首个 Tauri 2 重写版，基于 hgtuj5688/tvbox-desktop（Electron）全功能对齐：

- 订阅源管理：URL / 本地 JSON / 粘贴配置三种添加方式，刷新、激活、删除
- TVBox A/B 双格式兼容，站点类型矩阵（cms/api/html 支持，spider/unknown 灰显说明）
- 分类浏览 + 分页，站点连通性测试
- 聚合搜索：Rust tokio 并发（6 路）、单站 12s 超时、进度事件
- 详情：多播放源、选集、收藏
- 播放：ArtPlayer + hls.js，观看进度 8 秒节流记忆 + 自动续播
- 直播电视：m3u/txt 解析、同名频道多线路合并、并发测速
- 房间号：huya:// / douyin:// 实验性解析
- 简易嗅探：HTML 直取正则提取（主通道）+ Webview hook 窗口（增强通道）
- 首页推荐位：tvboxDesktop.home.sections，缺失时回退站点最新列表
- 三平台打包配置（NSIS / dmg / deb+AppImage），图标全套生成脚本
