# Rift 简体中文版

本仓库基于上游 [Rift v0.8.3](https://github.com/acsandmann/rift/tree/v0.8.3)（提交 `165e29a567f5858aee2ed09fe9e2233b50b55576`），提供简体中文的原生设置界面、菜单、总览、搜索和提示，并在“简单模式”下重做了窗口拖拽分区：候选位置只由屏幕空间决定，不再以当前窗口或窗口边缘为目标。

`origin` 指向自有仓库 `Promenar/rift`，`upstream` 指向 `acsandmann/rift` 只读来源；本地提交只进入自有仓库，不向上游推送或创建 PR。

## 分支

| 分支 | 内容 |
| --- | --- |
| `upstream-main` | 只读镜像，始终等于 `upstream/main`，不提交、不推送，只用于查看和合并上游 |
| `main` | 集成分支：上游最新 + 简体中文 + 本仓库功能改动 |
| `feat/*` | 功能分支，从 `main` 切出，验证通过后合入 `main` |
| `zh-cn/v0.8.3` | v0.8.3 汉化迁移的验证分支（已合入 `main`） |
| `zh-cn/v0.7.0` | v0.7.0 基线的汉化与多屏页签修复历史，保留用于对比和回退 |

## 使用范围

界面标题、设置说明、布局选项、规则与快捷键编辑器、搜索、菜单栏、总览和提示使用简体中文。配置键名、配置值、命令接口与默认快捷键沿用上游格式；用户填写的工作区名称、应用名称、路径、窗口标题和键位表名称保持原样。

快捷键页的动作名称来自 `#[command(label = …)]` 命令目录，已随目录本地化；方向与尺寸选项显示为「左 / 右 / 上 / 下」「水平 / 垂直」。上游终端帮助、配置示例注释、底层技术错误详情和日志保留英文。汉化不会自动改名已有工作区。

内建控件来自依赖 `cgs 0.5.2`。项目在 `vendor/cgs/` 固定其源码，仅汉化用户可见文案（按钮、导航、路径选择、快捷键录制与无障碍文字）；版本、依赖行为和 GPL-3.0 许可保持不变。来源归档的 SHA-256 记录在 `vendor/cgs/source-provenance.json`。

## 简单模式

在设置 › 通用中开启“简单模式”，或在 `~/.config/rift/config.toml` 中设置：

```toml
[settings]
simple_mode = true
```

简单模式在启动时生效，运行时修改需要重启 Rift。

### 工作区与布局

- 每块显示器只有一个工作区；不创建工作区、不切换、不显示工作区页。
- 布局固定为传统平铺；其他布局模式的选择入口隐藏，运行时切换命令不生效。
- 启动时忽略包含多工作区的旧布局快照；符合“每屏一个工作区且为传统平铺”的快照仍会恢复，页签组可跨重启保留。
- 显示器断开、重连或桌面编号变化时，空间迁移只保留每块显示器的一个工作区；其他工作区的窗口会并入保留的工作区，不会叠加成多个工作区。简单模式覆盖配置中 `mode = "global"` 的共享工作区行为。
- 关闭简单模式并重启 Rift 可恢复完整功能；简单模式期间合并掉的旧工作区不会自动恢复。
- **页签组尺寸修正**：页签组缩放后，未选中的成员会重新应用目标 frame，不再保留旧宽度从选中页签后面探出。
- **页签选中修复**：分区拖拽重建布局、以及从快照恢复后，每个页签组都会重新保证有一个本地选中的页签，标签栏高亮与前台可见窗口保持一致。

### 屏幕空间拖拽

拖拽候选只由**指针在显示器上的坐标**和**当前顶层组数量**决定，不读取任何窗口矩形，也不把指针下的窗口作为目标。简单模式没有旧窗口目标候选的回退路径；取不到分组信息时宁可不显示候选，也不会退回旧行为。

- **目标 2 组**：左右两列，全高候选。
- **目标 3–6 组**：三列 × 上/中/下三带。
  - 中带：候选为**整列全高**；
  - 上/下带：候选为该列的**上半/下半**，形成 1/4 或 1/6 屏目标。
- 拖拽时显示全屏分区网格：半透明暗色遮罩、蓝色网格线、淡蓝候选格；候选文字为 24px 粗体淡蓝色，居中显示在候选格内部，例如“全高 · 1/3 屏”“上半 · 1/4 屏”。
- 落点决定最终布局；空列自动收掉，非空列等宽；现有组按顺序填入其余格子。

#### 合并成页签组

- 中带落在**已有全高组**上时，拖动的窗口并入该组成为页签，而不是新建组。
- 源组还有剩余页签时，源组保留；源组只有被拖动的窗口时，源组消失。
- 合并后按新的组数重新排布。例如三个全高列时，把中列窗口拖到左列中带：左列变成两个页签，布局重排为 2 组左右两列。
- 上/下带不会触发合并，始终生成半高候选。

#### 四组布局

四组支持 `1/3 + 1/3 + 1/6 + 1/6`，半高列可以是左、中、右任意一列：

| 落点 | 结果 |
| --- | --- |
| 左列上/下带 | 左上/左下 + 中全高 + 右全高 |
| 中列上/下带 | 左全高 + 中上下 + 右全高 |
| 右列上/下带 | 左全高 + 中全高 + 右上/右下 |

中带仍保持目标列全高：左列/中列中带会把右列拆成上下，右列中带会把左列拆成上下。`2×2` 仍属于合法布局模型，但当前 3×3 网格尚未分配独立触发手势。

#### 五组与六组

- 5 组：三列中两列上下分 + 一列全高。
- 6 组：三列全部上下分。

### 其他行为

- 平铺窗口的边框拖拽缩放不会修改布局权重：拖拽期间按移动处理，松开后窗口回到原平铺 frame。
- 简单模式不再提供拖拽边缘插入/切分；切分修饰键不参与简单模式落点判定。
- 完整模式仍保留上游的窗口目标拖拽、边缘插入和切分修饰键行为。

## 应用规则

- **顺序即优先级**：规则从上到下匹配，第一条启用的命中规则生效；更具体的规则（例如按标题或 AX 角色）必须排在通用规则之前。
- **启用开关**：每条规则有 `enabled`（默认 `true`），列表行尾和详情页都可切换；切换后立即对活动空间的窗口重新套用。
- **从窗口取数**：设置页“添加”会列出当前打开的窗口（应用、标题、工作区、AX 角色、窗口标签）。选择后生成一条复现该窗口现状的规则；生成的规则不包含原始窗口标签，需要时可在详情页手动勾选。
- **窗口标签匹配**：`window_tags` 要求所选位全部存在，但全屏能力位（`FULL_SCREEN_CAPABLE`、`FULL_SCREEN_TILE_CAPABLE`）会被忽略——macOS 在窗口挂载 sheet 时会临时清掉它们。
- **命中预览**：规则页底部按当前顺序显示每个窗口命中的规则序号，或“未命中”。
- **忽略规则**：管理方式选“忽略”后会清空并禁用该规则的工作区、浮动、尺寸与聚焦设置。
- 命令行：`rift-cli query rule-windows` 返回同样的窗口与匹配字段，便于排查。

## 安装状态与回退

2026-11-18 首次安装 v0.8.3 中文版，之后多次原位更新；当前为 2026-11-24 的屏幕空间分区拖拽版本：

- 版本目录：`~/Library/Application Support/Rift/Versions/0.8.3-zh-cn-20261118-022151`
- `/opt/homebrew/bin/rift` 与 `rift-cli` 指向该目录，`rift --version` 输出 `0.8.3+701cd5075438`
- 二进制 SHA-256：`1cc8aab2a747d5fe35fb37d7c317fa1e2b12342fab954c8c168d3be0b4fcb726`
- 二进制已用本机 `Apple Development` 证书签名，designated requirement 为「标识符 + 证书」，不绑定 cdhash；用同一身份重新签名并替换后，辅助功能授权可继续沿用。
- 服务用 `rift service restart` 管理；`rift-cli service` 已提示迁到 `rift`。
- 用户配置、快捷键、应用规则与 LaunchAgent 未改；持久化布局快照在升级前备份。

安装过程中的已知现象：

- 替换二进制后的首次启动可能被 macOS 以 `SIGKILL (Code Signature Invalid) / Launch Constraint Violation` 终止一次，launchd 自动重启后稳定。
- 屏幕休眠/锁定期间重启后，`rift-cli query workspaces` 可能只看到当前工作区；唤醒后再次 `rift service restart` 可恢复。
- 全屏应用（游戏、视频）会占用独立 macOS 空间，此时查询只显示该空间的工作区，属正常现象。

回退到 v0.7.2 中文版：

```sh
BK="$HOME/Library/Application Support/Rift/Backups/20261118-022151-v083-authorized"
python3 "$BK/rollback.py" --check
python3 "$BK/rollback.py"
```

备份目录同时保存升级前的 LaunchAgent、`config.toml`、`layout.ron`、旧入口链接与本机校验值。`rollback.py` 会校验旧二进制 SHA-256 后再切回，并按目标版本选择 `rift service` 或 `rift-cli service`。

更早的备份保留在 `~/Library/Application Support/Rift/Backups/`：
`20261027-091741-tab-coordinate`（0.7.0 页签修复版）、`20261019-192206`（首次中文版）、
`20261102-005502-v072-authorized`（0.7.2 回退点），以及
`20261123-203359-simple-reconnect-fix`（简单模式拓扑修复前的二进制与布局快照）。

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

`settings_native` 需要真实 AppKit 窗口，上游 CI 明确跳过它；本地通过 `--keyboard`、`--about` 和 `--render` 截图单独验证。

2026-11-24 在当前分支上的结果：

| 验证 | 结果 |
| --- | --- |
| `cargo +nightly fmt --all --check` | 通过 |
| `cargo check --locked -j 2` | 通过 |
| `cargo nextest run --locked -E 'not binary(settings_native)' -j 2` | 941 项通过，0 失败 |
| zone 专项测试 | 14 项通过：屏幕格几何、2/3/4 组布局、合并目标、传统树重建、页签选中修复 |
| arm64 release 构建 | `rift` 与 `rift-cli` 生成成功 |
| 简单模式实拖（2026-11-24） | 网格持续显示；中带全高、上/下半高可稳定触发；中列窗口拖到左列中带会合并为页签组并重排为两组；候选文字居中、24px 粗体淡蓝色 |
| 安装运行 | `rift --version` 为 `0.8.3+701cd5075438`；两块显示器各只有一个工作区；辅助功能授权保留 |

已知问题：`settings_native --ignored --verify` 在 `search replaces the sidebar` 断言失败，**纯上游 v0.8.3 在同一台机器上同样失败**，与本仓库汉化无关；该断言依赖搜索态下侧栏视图被替换的时序。快捷键与关于页另有独立入口，已单独通过。

构建产物位于 `target/aarch64-apple-darwin/release/`。

## 许可

Rift 代码沿用 [Apache License 2.0](LICENSE)。`cgs` 保留其独立的 [GPL-3.0 许可](vendor/cgs/LICENSE)，未将其改标为 Apache。
