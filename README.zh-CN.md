# Rift 简体中文界面

本目录基于 [Rift v0.7.0](https://github.com/acsandmann/rift/tree/v0.7.0)，提供简体中文的原生设置界面、菜单、总览和提示，以及多屏页签坐标修复。

## 使用范围

界面标题、设置说明、布局选项、规则与快捷键编辑器、搜索和提示使用简体中文。配置键名、配置值、命令接口与默认快捷键沿用上游格式；用户填写的工作区名称、应用名称、路径和窗口标题保持原样。

上游终端帮助、配置示例注释和底层技术错误详情可保留英文。汉化不会自动改名已有工作区。

内建控件来自已有依赖 `cgs 0.3.9`。项目在 `vendor/cgs/` 固定其源码，仅汉化动态录制提示、取消与完成按钮、路径选择、导航和无障碍文字；版本及依赖保持一致。来源归档的 SHA-256 与原许可随源码保留。

## 设置入口

本机已安装并运行中文版，通过终端打开设置：

```sh
rift-cli settings
```

`/opt/homebrew/bin/rift` 与 `rift-cli` 指向用户 `Library/Application Support/Rift/Versions/0.7.0-zh-cn-tab-coordinate-20261027-091741/bin/` 的独立安装副本。Homebrew Cellar 原文件和首次中文安装副本均保留。中文版外观可查看[布局页截图](.work/previews/layouts.png)和[菜单栏页截图](.work/previews/menu-bar.png)。

## 多屏页签坐标

适用环境：Desktop / macOS，多块屏幕的高度或上下排列不同。全局 Cocoa/Quartz 坐标转换从同一显示器的两套坐标推导翻转常量，避免高外屏或下置外屏将主屏页签推到屏幕之外。未改变拖放规则、页签位置配置、窗口分组或快捷键。

2026-10-27 已在本机安装修复版。回归测试先复现218像素偏移；修复后10项屏幕测试、865项完整单元测试和arm64 release构建通过，独立源码审阅无阻断项。真实WindowServer页签窗口从Y=-184恢复到Y=34，高28、透明度1且处于屏内显示状态；两屏布局树及全部运行配置保持一致。用户已确认顶部页签恢复。CUA不能访问当前Codex窗口，未采集验收截图，也未进行真实竖屏热插拔验收。

修复方案见 [.agents/plans/2026-10-27-tab-coordinate-fix.md](.agents/plans/2026-10-27-tab-coordinate-fix.md)。本机备份位于 `~/Library/Application Support/Rift/Backups/20261027-091741-tab-coordinate/`，包含程序、配置、启动定义、运行布局、安装校验和运行坐标校验。回退到此前中文版：

```sh
python3 "$HOME/Library/Application Support/Rift/Backups/20261027-091741-tab-coordinate/rollback.py" --check
python3 "$HOME/Library/Application Support/Rift/Backups/20261027-091741-tab-coordinate/rollback.py"
```

回滚保留当前配置和快捷键。若要恢复最初Homebrew原版，先执行上述回退，再执行下一节首次安装备份的回滚命令。

## 配置、模拟器与备份

安装切换保留原配置与快捷键。按用户要求，应用规则新增 `app_name = "qemu-system-aarch64"`、`manage = false`，精确排除本机 Android 模拟器；没有使用会影响 Android Studio 的名称匹配。两个模拟器窗口均已从管理树中移除。新增该规则时，其余运行配置与原配置完全一致，快捷键及快捷键模式未变。

随后观察到 `settings.drag_drop` 发生更新，已保留最新文件，没有用备份覆盖。服务、IPC与原工作区成员恢复检查通过。启动定义已规范化为合法XML，并使用 `--restore` 恢复保存的布局；辅助功能权限由用户手动授予。

原二进制、配置、启动定义及运行布局备份位于本机：

```text
~/Library/Application Support/Rift/Backups/20261019-192206/
```

回滚检查与恢复命令：

```sh
python3 "$HOME/Library/Application Support/Rift/Backups/20261019-192206/rollback.py" --check
python3 "$HOME/Library/Application Support/Rift/Backups/20261019-192206/rollback.py"
```

回滚默认保留当前配置和快捷键，仅恢复原版入口与启动定义。备份中的 `installation-receipt.json`、`emulator-exclusion-receipt.json` 记录实际安装校验与规则边界；含私有配置的备份目录权限为700，不进入Git。

实际设置子进程已启动过。CUA无法绑定这个无App Bundle的命令行程序，真实窗口及窄尺寸未完成自动UI验收，保留给用户试用。Homebrew升级可能重建命令入口，独立中文版副本和备份不会因此自动删除。

## 构建与验证

此工程依赖 macOS 原生框架。候选使用本机 macOS arm64、已有 Rust/Cargo 和 Xcode，并锁定 `Cargo.lock`；执行方案和授权状态见 [.agents/plans/2026-10-19-zh-cn-ui.md](.agents/plans/2026-10-19-zh-cn-ui.md)。

本机候选已编译，并于2026-10-19完成服务切换。构建验证环境为 Desktop / macOS 27.0.1 arm64、Xcode 27.1、Rust/Cargo 1.98.0。

| 验证 | 实际结果 |
| --- | --- |
| `cargo check --locked -j 2` | 通过 |
| `cargo test --locked --lib -j 2` | 862 项通过，0 失败 |
| `cargo test --locked --test settings_native -j 2 -- --ignored --verify` | 原生控件、配置提交与同步、更新状态和关闭检查通过 |
| arm64 release 构建 | `rift` 与 `rift-cli` 均成功生成 |
| 可执行文件检查 | 两个文件均为 arm64，`--version`、`--help`、`codesign --verify --strict` 通过 |
| 构建阶段只读 IPC | 候选 CLI 成功查询服务，返回合法工作区与窗口数据；验证凭据仅记录数量 |
| 有效隔离界面截图 | 11 个设置页、3 组紧凑窗口和 2 组深色界面，共 16 组 |

700 像素窄窗口的主从布局和展开间距输入均通过布局断言；路径选择按钮的中文标题有原生回归检查。截图来自与候选相同源码的隔离 AppKit 测试程序，未接管当前桌面的窗口管理。

额外的 700 像素键盘页截图在内容区域出现透明像素；880 像素标准页正常。对照排除了高度、祖先隐藏与透明度，仍未确认隐藏窗口绘制路径的内部根因。该压力用例未计入有效截图，不代表实际显示窗口已通过窄尺寸验收；诊断未修改生产布局。

构建命令：

```sh
cargo build --release --locked -j 2 --package rift-wm --bin rift --bin rift-cli --target aarch64-apple-darwin
```

候选文件位于 `target/aarch64-apple-darwin/release/`，版本标识为 `0.7.0+dea7a23ed6d2.dirty`，表示上游基线上的本地修改。下列SHA-256属于2026-10-19首次中文构建；当前页签修复版的构建及签名哈希以修复备份中的 `installation-receipt.json` 为准：

| 文件 | SHA-256 |
| --- | --- |
| `rift` | `50106518333b5e826f7f4ac143e28c0bac527ad92bd9013bf1473ae13e482b92` |
| `rift-cli` | `fc089a4baa184667227edf13b52043e96f1b8f643352eb9482b99c50dd3831e6` |

详细凭据位于 [`.work/validation-receipt.json`](.work/validation-receipt.json)，截图清单位于 [`.work/previews/receipt.json`](.work/previews/receipt.json)，原始日志位于 `.work/logs/`。这些本机可复现产物由 Git 忽略。

表中哈希属于构建产物；独立安装副本按Homebrew公式执行本机签名后，哈希另见安装凭据。当前中文服务运行与只读IPC检查通过；真实窗口的完整交互与长期稳定性由实际试用补充。已有工具链缺少 rustfmt 组件，因此未运行格式化；编译、测试和 diff 空白检查已通过。

## 许可

Rift 代码沿用 [Apache License 2.0](LICENSE)。`cgs` 保留其独立的 [GPL-3.0 许可](vendor/cgs/LICENSE)，未将其改标为 Apache。中文界面相关文件的修改说明见 Git diff 和 [.agents/handover.md](.agents/handover.md)。
