# Rift 简体中文界面

本仓库基于上游 [Rift v0.8.3](https://github.com/acsandmann/rift/tree/v0.8.3)（提交 `165e29a567f5858aee2ed09fe9e2233b50b55576`），提供简体中文的原生设置界面、菜单、总览、搜索和提示。

仓库由上游原作者仓库 Fork 而来。`origin` 指向自有仓库 `Promenar/rift`，`upstream` 指向 `acsandmann/rift` 只读来源；本地提交只进入自有仓库，不向上游推送或创建 PR。

## 分支

| 分支 | 内容 |
| --- | --- |
| `upstream-main` | 只读镜像，始终等于 `upstream/main`，不提交、不推送，只用于查看和合并上游 |
| `main` | 集成分支：上游最新 + 简体中文 + 本仓库功能改动 |
| `feat/*` | 功能分支，从 `main` 切出，验证通过后合入 `main` |
| `zh-cn/v0.8.3` | v0.8.3 汉化迁移的验证分支（已合入 `main`） |
| `zh-cn/v0.7.0` | v0.7.0 基线的完整汉化与多屏页签修复历史，保留用于对比和回退 |

## 使用范围

界面标题、设置说明、布局选项、规则与快捷键编辑器、搜索、菜单栏、总览和提示使用简体中文。配置键名、配置值、命令接口与默认快捷键沿用上游格式；用户填写的工作区名称、应用名称、路径、窗口标题和键位表名称保持原样。

快捷键页的动作名称来自 `#[command(label = …)]` 命令目录，已随目录本地化；方向与尺寸选项显示为「左 / 右 / 上 / 下」「水平 / 垂直」。上游终端帮助、配置示例注释、底层技术错误详情和日志保留英文。汉化不会自动改名已有工作区。

内建控件来自依赖 `cgs 0.5.2`。项目在 `vendor/cgs/` 固定其源码，仅汉化用户可见文案（按钮、导航、路径选择、快捷键录制与无障碍文字）；版本、依赖行为和 GPL-3.0 许可保持不变。来源归档的 SHA-256 记录在 `vendor/cgs/source-provenance.json`。

## 与 v0.7.2 汉化的差异

- 上游 75 个提交带入的界面按 v0.8.3 结构重新汉化：重写的快捷键页（动作目录、键位表管理、命令参数编辑器）、应用规则的原始窗口标签编辑器、Mission Control 新增选项和菜单栏显示方式选择器。
- 快捷键动作目录从页面内的本地列表迁到 `rift-protocol` 与 `wm_controller` 的 `#[command(label)]` 属性；所有动作项已补齐中文标题，原先依赖自动英文命名的项也显式标注。
- 命令参数编辑器补充中文标签：方向、水平/垂直、显示器、文件、数值、增量等；未列出的字段回退为按字段名生成的英文标签。
- `cgs` 由 0.4.0 升至 0.5.2，新增的快捷键录制与快捷键设置组件文案已本地化，原有 39 条对照全部沿用。
- 上游把默认工作区模式改为 `global`（所有显示器共用一组工作区）。本仓库**沿用上游默认值**，未改回 `per_display`；需要旧行为时在配置中设置 `workspace_mode = "per_display"`。
- 上游把测试固定为单线程（`.cargo/config.toml` 设置 `RUST_TEST_THREADS=1`），本仓库保留该设置。

## 本仓库的功能改动

以下行为与上游不同，属于本仓库的分叉功能。

### 应用规则

- **顺序即优先级**：规则从上到下匹配，第一条启用的命中规则生效。原先“匹配字段越多越优先”的规则已移除；更具体的规则（例如按标题或 AX 角色）必须排在通用规则之前。
- **启用开关**：每条规则有 `enabled`（默认 `true`）。规则列表行尾和详情页都可切换；切换后立即对所有活动空间的窗口重新套用，无需重启。停用的规则会被跳过，也不会触发窗口标签查询。
- **从窗口取数**：设置页“添加”会列出当前打开的窗口（应用、标题、工作区、AX 角色、窗口标签）。选择后自动生成一条复现该窗口现状的规则：应用、标题子串、AX 角色、工作区、浮动与尺寸；生成的规则不包含原始窗口标签（该字段会随弹层临时变化），需要时可在详情页“原始窗口标签”中手动勾选。之后可在详情页改成只按应用匹配。
- **窗口标签匹配**：`window_tags` 要求所选位全部存在，但全屏能力位（`FULL_SCREEN_CAPABLE`、`FULL_SCREEN_TILE_CAPABLE`）会被忽略——macOS 在窗口挂载 sheet 时会临时清掉它们，否则规则会在打开弹窗时突然失效、窗口被重新接管并放大。
- **命中预览**：规则页底部按当前顺序显示每个窗口命中的规则序号，或“未命中”。
- **忽略规则**：管理方式选“忽略”后会清空并禁用该规则的工作区、浮动、尺寸与聚焦设置，避免与“不管理该窗口”冲突；配置校验文案也已本地化。
- 命令行：`rift-cli query rule-windows` 返回同样的窗口与匹配字段，便于排查。

### 拖拽叠组

- 指针落在窗口之间的**间隙**或堆叠组的**标签栏**上时，会归入 40pt 内的最近窗口并按“放到中心时”的动作处理，不再落空或误判为插入。
- 目标窗口的**整块区域**默认执行 `drop_action`（默认堆叠/交换）；只有按住 `split_modifier`（默认 `shift`）时，边缘区域才变成插入/切分。
- 源窗口腾空位的一步移动只在附近没有目标时生效；迟滞改为“仍在原目标上且没有其他窗口覆盖”，换目标更跟手。
- 按 `drop_zone_fraction`（默认 25%）控制切分边缘的宽度。
- 切分修饰键在设置页“鼠标与触控板 › 拖放 › 切分修饰键”中修改，语法与拖拽修饰键相同。

## 安装状态与回退

2026-11-18 首次安装 v0.8.3 中文版，2026-11-19 原位更新：

- 版本目录：`~/Library/Application Support/Rift/Versions/0.8.3-zh-cn-20261118-022151`
- `/opt/homebrew/bin/rift` 与 `rift-cli` 指向该目录，`rift --version` 输出 `0.8.3+213e7971c543`
- 二进制 SHA-256：`53590f92cbe21542fd0b46491c055e89ff8cb280ebe3ccfee11ddf5402f99c3a`
- 二进制已用本机 `Apple Development` 证书签名，designated requirement 为「标识符 + 证书」，不再绑定 cdhash；用同一身份重新签名并替换后，辅助功能授权可继续沿用（2026-11-18 已实测：重建后 cdhash 变化，窗口接管无需重新授权）
- 服务用 `rift service restart` 管理；`rift-cli service` 已提示迁到 `rift`
- 用户配置、快捷键、应用规则与 LaunchAgent 未改；持久化布局快照在升级前备份

安装过程中的已知现象：

- 替换二进制后的首次启动被 macOS 以 `SIGKILL (Code Signature Invalid) / Launch Constraint Violation` 终止一次，launchd 自动重启后稳定；之后 `rift service restart` 未复现，也未新增崩溃报告。
- 屏幕休眠/锁定期间重启后，`rift-cli query workspaces` 只看到当前工作区；唤醒后再次 `rift service restart` 可恢复完整工作区。全屏应用（游戏、视频）会占用独立 macOS 空间，此时查询同样只显示该空间的工作区，属正常现象；两种情况都未进一步定位根因。
- 上游 global 模式下 Rift 会为没有工作区的显示器自动播种一个工作区，因此本机现在除 first/second/Workspace 3/Workspace 4 外多出一个 Workspace 5（位于第二台显示器）。需要旧的 per-display 行为时，在 `~/.config/rift/config.toml` 的 `[virtual_workspaces]` 中设置 `mode = "per_display"`。

回退到 v0.7.2 中文版：

```sh
BK="$HOME/Library/Application Support/Rift/Backups/20261118-022151-v083-authorized"
python3 "$BK/rollback.py" --check
python3 "$BK/rollback.py"
```

备份目录同时保存升级前的 LaunchAgent、`config.toml`、`layout.ron`、旧入口链接与本机校验值。`rollback.py` 会校验旧二进制 SHA-256 后再切回，并按目标版本选择 `rift service` 或 `rift-cli service`。

## 本地开发签名

ad-hoc 签名的 designated requirement 是 cdhash，每次重建都会让辅助功能授权失效。本仓库改用本机开发证书签名，让授权跨重建继承：

```sh
cargo build --release --locked -j 2 --package rift-wm --bin rift --bin rift-cli --target aarch64-apple-darwin
scripts/local-sign.sh target/aarch64-apple-darwin/release/rift target/aarch64-apple-darwin/release/rift-cli
```

`scripts/local-sign.sh` 默认取 `security find-identity -p codesigning` 的第一个身份，可用 `RIFT_SIGN_IDENTITY` 覆盖。签名后核对：

```sh
codesign -d -r- ./rift   # 应显示 identifier "git.acsandmann.rift" 与证书 CN，而非 cdhash
```

注意：只有同一签名身份、同一二进制路径才能继承授权；换证书、换路径或改用 ad-hoc 签名都需要重新授权。

回退到安装前的 0.7.0 中文版：

```sh
python3 "$HOME/Library/Application Support/Rift/Backups/20261102-005502-v072-authorized/rollback.py" --check
python3 "$HOME/Library/Application Support/Rift/Backups/20261102-005502-v072-authorized/rollback.py"
```

更早的备份保留在 `~/Library/Application Support/Rift/Backups/`：
`20261027-091741-tab-coordinate`（0.7.0 页签修复版）、`20261019-192206`（首次中文版）以及
`20261102-000648-v072-install`、`20261102-001409-v072-inplace`（两次被辅助功能授权拦截的安装尝试）。

## 构建与验证

依赖 macOS 原生框架，使用本机 macOS arm64、Rust/Cargo 与 Xcode。开发环境另需：

- `nightly` 工具链的 `rustfmt`：`rustup toolchain install nightly --profile minimal --component rustfmt`。`rustfmt.toml` 使用 `unstable_features` 等 nightly 专有选项，stable rustfmt 无法执行。
- `cargo-nextest`：`cargo install cargo-nextest --locked`，用于复现上游 CI 的测试命令。
- `clippy`（可选）：`rustup component add clippy`。上游 CI 不运行 clippy，仅作为本地增强。

验证命令（与上游 CI 对齐）：

```sh
cargo +nightly fmt --all --check
cargo check --locked -j 2
cargo nextest run --locked -E 'not binary(settings_native)' -j 2
cargo test --locked --test settings_native -j 2 -- --ignored --keyboard
cargo test --locked --test settings_native -j 2 -- --ignored --about
cargo build --release --locked -j 2 --package rift-wm --bin rift --bin rift-cli --target aarch64-apple-darwin
```

`settings_native` 需要真实 AppKit 窗口，上游 CI 明确跳过它（`-E 'not binary(settings_native)'`）；本地通过 `--keyboard`、`--about` 和 `--render` 截图单独验证。

2026-11-18 在 v0.8.3 基线上的实际结果：

| 验证 | 结果 |
| --- | --- |
| `cargo check --locked -j 2` | 通过 |
| `cargo test --locked --lib -j 2` | 917 项通过，0 失败 |
| `cargo nextest run -E 'not binary(settings_native)'` | 920 项通过，0 失败 |
| `settings_native --ignored --keyboard` | 快捷键录制、冲突处理、目录选择通过 |
| `settings_native --ignored --about` | 更新状态与请求清理通过 |
| 设置页截图（`--render --page 0..10`） | 11 个页面均为中文渲染；规则页显示行内开关与命中预览；鼠标页显示切分修饰键 |
| arm64 release 构建 | `rift` 与 `rift-cli` 生成成功 |
| 安装运行（2026-11-19） | `rift --version` 为 `0.8.3+213e7971c543`；`rift-cli query rule-windows` 返回真实窗口与 AX 角色；签名重建后辅助功能授权仍有效；打开规则编辑 sheet 后忽略规则的窗口保持原尺寸、不重新接管 |

已知问题：`settings_native --ignored --verify` 在 `search replaces the sidebar` 断言失败，**纯上游 v0.8.3 在同一台机器上同样失败**，与本仓库汉化无关；该断言依赖搜索态下侧栏视图被替换的时序。快捷键与关于页另有独立入口，已单独通过。

构建产物位于 `target/aarch64-apple-darwin/release/`。`cargo +nightly fmt --all --check`、`cargo check --locked`、单元测试与 nextest 均已通过；原生快捷键/关于验证和页面截图按上文命令单独执行。

## 许可

Rift 代码沿用 [Apache License 2.0](LICENSE)。`cgs` 保留其独立的 [GPL-3.0 许可](vendor/cgs/LICENSE)，未将其改标为 Apache。
