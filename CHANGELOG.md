# 更新日志

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
