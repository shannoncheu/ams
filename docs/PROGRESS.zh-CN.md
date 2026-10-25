# Slate / Portable 验证状态

本版本保留上游 gpui-apple-music-demo 的音频、歌词、曲库和队列实现，使用固定的 GPUI Fusion 提交。当前以 Windows 10 x64 为目标，真实实机验收尚未完成。

## 当前实现

- 深灰蓝主背景 #20252B、面板 #303840、冰蓝强调色 #91C8D5，保留封面/歌词/控制区位置。
- 右上角常驻最小化、最大化/还原、关闭图标；无默认按钮底色，悬停使用轻微透明底色，关闭悬停为柔和红色。保留 GPUI/Win32 原生拖动、双击最大化、边缘缩放与系统贴靠路径，不使用全屏模拟最大化。
- Noto Sans SC 可变字体嵌入 EXE，Regular/Medium/Semibold 分层，保留 SIL OFL 1.1。歌词提高邻近行可读性，当前行逐字高亮，沿用平滑滚动和渐隐。
- Acrylic 可选：检测 Windows 版本、桌面合成、系统透明设置、远程会话及 API 返回值；不可用时使用不透明深灰蓝，不依赖 Windows 11 Mica。
- 便携标记 portable.flag 或 --portable 将配置、状态、日志和缓存放入 EXE 旁的 data/，不改 HOME；普通模式保留原路径兼容。
- 队列封面使用 96px 后台缩略图，按视口保留不超过 64 张，最多两个实际解码任务，设置尺寸/分配上限并缓存失败结果，避免循环重试。主封面文件准备也移出 UI 更新闭包。
- 歌词字号过渡复用端点字形，再进行短暂缩放，减少重复创建字形资源；暂停且动画收敛后停止请求动画帧。两个 Tokio 工作线程，音频解码引擎未重写。

## 已执行检查

| 检查 | 结果与范围 |
| --- | --- |
| Linux cargo check / build --locked | 通过；不代表 Windows 编译 |
| Linux cargo test --locked | 22 项单元测试与 1 项独立目录来源集成测试通过 |
| rustfmt / git diff --check | 通过 |
| PowerShell 脚本 AST 解析 | 通过；不代表已在 Windows 执行 |
| Windows Server 2022 | [首轮真实 Actions](https://github.com/shannoncheu/ams/actions/runs/40162318865) 检查、测试、Release 编译及 Portable 窗口/配置/退出检查通过；最终包将纳入后续许可证补全 |
| Wayland / 软件 Vulkan | 原生 GPUI 实际启动、中文字体、主窗/迷你窗切换、位置保持已检查 |
| Linux Portable 路径 | 独立 EXE 目录加标记启动 --smoke-test，正常退出，在 data/config 创建配置 |
| 许可证收集 | 已从锁定的 Windows 依赖图收集 491 个包的声明与可用许可文件，并包含字体 OFL |
| 敏感文件检查 | 待上传源码未发现私钥或常见 Token；运行数据、缓存、日志、.env 和凭据文件排除 |

[主窗截图](images/player-preview.png) · [迷你窗截图](images/mini-preview.png)

截图来自 Linux 软件渲染，使用独立测试素材，不能作为 Windows 字体渲染、透明特效或硬件性能验收。

## Windows 构建与验收边界

GitHub Actions 使用 windows-2022（Windows Server 2022），执行锁定依赖检查/测试、MSVC x64 Release、静态 CRT 请求、DLL 依赖检查、许可证收集及 EXE/普通 ZIP/Portable ZIP 打包。Portable 在新目录解压，从无关工作目录启动，检查退出码、真实窗口和本地配置生成；Artifacts 保存实际报告。

Windows Server 构建或启动成功也不等于 Windows 10 兼容性通过。真实 Win10 播放、休眠/设备切换、Acrylic/透明降级、任务栏范围、Snap、多屏/DPI、字体外观以及长期 CPU/GPU/RAM 仍需实机验证。当前不声称满足约 100 MB 或稳定 60 FPS。CI 工作集峰值只作为服务器启动遥测，GPU 未测时标记为空。

Linux 虚拟音频曾成功录到非零 PCM，但复测间歇出现 ALSA/Pulse I/O 错误；原因未定，音频稳定性未验收。

YouTube Music 仅有来源接口和边界设计，Google 登录、在线元数据、远端歌单及原生播放均未接通。迷你展开/收起动画、可切换置顶、完整背景模式及大型媒体库索引仍待完善。发布为无签名预览，不作为正式稳定版。

首轮 CI 启动工作集峰值为 172,625,920 字节（约 164.6 MiB），不是本地播放常驻内存；CPU/GPU 尚未测量。导入表包含系统 icuuc.dll，因此保守要求 Windows 10 1903 或更新版本（建议 22H2），旧版本不在当前支持声明内。
