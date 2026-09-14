# VarMan

[English](./README.en-US.md) | 简体中文

VarMan 是一个跨平台系统环境变量可视化管理工具，用于替代 Windows、Linux 和 macOS 自带的环境变量编辑界面。它基于 Tauri 2、React 19、TypeScript、Tailwind CSS 4 和 shadcn/ui 构建，提供统一的列表视图、变更预览、自动备份、恢复和权限保护。

> 当前版本为 `0.1.0`，功能已覆盖核心管理流程，但仍建议在生产环境中谨慎修改系统变量。

## 目录

- [功能亮点](#功能亮点)
- [平台行为](#平台行为)
- [安全模型](#安全模型)
- [备份与恢复](#备份与恢复)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [常用命令](#常用命令)
- [项目结构](#项目结构)
- [开发说明](#开发说明)
- [测试与验证](#测试与验证)
- [打包](#打包)
- [持续集成](#持续集成)
- [常见问题](#常见问题)

## 功能亮点

### 变量管理

- 查看用户级和系统级环境变量，支持按名称实时搜索。
- 显示每个变量的来源：Windows 显示注册表，Unix 显示具体配置文件路径。
- 新增、编辑和删除环境变量。
- 值包含 `;` 时自动切换为逐条列表编辑，可添加、删除和单独修改每个条目。
- 单值和多值模式都支持手动输入，也可通过系统对话框选择目录或文件。
- Windows 可保留 `REG_EXPAND_SZ` 引用，读取时展示展开值，写入时尽量保留原始引用。

### 变更预览

- 所有写入操作前都会生成 diff 预览。
- diff 使用“修改前 / 修改后”双列展示。
- `;` 分隔的值会拆成条目并逐行对比，新增、删除和修改的条目分别用绿色、红色标识。
- 未变化的条目保持中性样式，便于关注真正受影响的内容。

### Path 专用编辑器

- Path 变量以专用抽屉打开。
- 每个路径条目独立展示，支持拖拽排序。
- 自动标记缺失目录和重复路径。
- 底部实时预览拼接后的完整 Path。
- 支持逐条删除和手动添加。

### 界面与语言

- 深浅色主题适配。
- 中英文界面，支持即时切换并持久化语言选择。
- 无权限时自动进入只读模式，并给出提权指引。

## 平台行为

### Windows

| 项目 | 说明 |
| --- | --- |
| 用户变量 | `HKEY_CURRENT_USER\Environment` |
| 系统变量 | `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` |
| 变量类型 | 支持 `REG_SZ` 和 `REG_EXPAND_SZ` |
| 引用保留 | `REG_EXPAND_SZ` 原始引用会被保存并在写入时恢复 |
| 写入通知 | 写入后广播 `WM_SETTINGCHANGE` |
| 提权方式 | 系统变量只读时支持按需以管理员身份重启 |
| 备份格式 | JSON 快照 |
| 备份目录 | `%APPDATA%\VarMan\backups` |

### Linux / macOS

| 作用域 | 配置文件 |
| --- | --- |
| 用户变量 | 优先使用 `~/.zshrc`；不存在时依次尝试 `~/.bashrc`、`~/.bash_profile`、`~/.profile` |
| 系统变量 | `/etc/environment`、`/etc/profile` |

Unix 实现特点：

- 解析 `export VAR=value` 和 `VAR=value` 两种写法。
- 忽略注释行和空行。
- 写入时先修改内存内容，再写入临时文件并原子替换。
- 值包含空格或特殊字符时使用双引号并转义内部双引号。
- 系统级写入要求进程以 root 身份运行。
- 备份目录为 `~/.VarMan/backups`。

## 安全模型

1. **必须预览**：`apply_changes` 和恢复备份前都会先生成 diff。
2. **权限检查**：写入前检查当前进程是否有目标作用域的写权限。
3. **自动备份**：实际写入前创建备份，无需用户手动执行。
4. **原子写入**：Unix 配置文件使用临时文件加原子替换，降低写坏文件的风险。
5. **只读降级**：权限不足时界面自动只读，避免误操作。
6. **恢复保护**：只能恢复 VarMan 备份目录中的备份文件。

错误会以结构化对象返回，包含稳定的错误码和消息。前端根据错误码显示对应提示，常见错误码如下：

| 错误码 | 含义 |
| --- | --- |
| `PERMISSION_DENIED` | 权限不足，需要提权或使用管理员/root 身份运行 |
| `NOT_FOUND` | 环境变量或文件不存在 |
| `PARSE_ERROR` | 配置文件或请求格式错误 |
| `IO_ERROR` | 系统读写失败 |
| `UNSUPPORTED` | 当前平台不支持该操作 |

## 备份与恢复

- 每次应用变更前都会自动备份目标作用域。
- 手动创建备份时也可选择用户或系统作用域。
- 备份列表显示路径、创建时间和作用域。
- 恢复前会生成差异预览，确认后才会写入。
- 恢复操作本身也会先创建新的备份。
- Windows 使用 JSON 快照；Unix 备份所有存在的候选配置文件。

## 环境要求

| 依赖 | 版本建议 |
| --- | --- |
| Node.js | 22 或更新版本 |
| pnpm | 10 或更新版本 |
| Rust | stable 工具链 |
| Windows | Windows 10 或更新版本，安装 WebView2 Runtime |
| Linux | WebKitGTK 4.1 及 Tauri 2 相关系统依赖 |
| macOS | Xcode Command Line Tools |

Ubuntu/Debian 依赖安装示例：

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

## 快速开始

```bash
# 安装前端依赖
pnpm install

# 启动桌面开发模式
pnpm tauri dev
```

开发模式下 Vite 使用固定端口 `1420`。如果端口被占用，命令会失败，需要先释放端口。

## 常用命令

| 命令 | 说明 |
| --- | --- |
| `pnpm dev` | 仅启动 Vite 前端开发服务器 |
| `pnpm tauri dev` | 启动完整 Tauri 桌面应用 |
| `pnpm build` | TypeScript 检查并构建前端产物 |
| `pnpm preview` | 预览前端构建产物 |
| `pnpm tauri build` | 构建当前平台安装包 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 运行 Rust 单元测试 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | 检查 Rust 格式 |

## 项目结构

```text
VarMan/
├── .github/workflows/
│   └── build.yml                  # 三平台 CI 构建工作流
├── docs/
│   ├── prompt.txt                 # 原始产品需求
│   └── superpowers/               # 设计与实施计划
├── src/
│   ├── App.tsx                    # 应用布局与编辑流程编排
│   ├── components/
│   │   ├── BackupDialog.tsx       # 备份与恢复
│   │   ├── DiffDialog.tsx         # 双列 diff 预览
│   │   ├── PathEditor.tsx         # Path 专用编辑器
│   │   ├── PermissionAlert.tsx    # 权限提示与提权
│   │   ├── Sidebar.tsx            # 作用域与语言切换
│   │   ├── Toolbar.tsx            # 搜索与操作入口
│   │   ├── VariableDialog.tsx     # 新增/编辑变量
│   │   ├── VariableTable.tsx      # 变量列表
│   │   └── ui/                    # shadcn/ui 组件
│   ├── lib/
│   │   ├── api.ts                 # Tauri command 封装
│   │   ├── errors.ts              # 错误映射
│   │   └── i18n.ts                # i18next 初始化
│   ├── locales/                   # 中英文语言包
│   ├── stores/env-store.ts        # Zustand 状态管理
│   └── types/env.ts               # 前端共享类型
└── src-tauri/
    ├── capabilities/default.json  # Tauri 权限配置
    ├── src/commands/mod.rs        # Tauri command 入口
    ├── src/core/                  # 模型、错误、diff、Path 逻辑
    ├── src/platform/              # Windows 与 Unix 实现
    └── tauri.conf.json            # Tauri 主配置
```

## 开发说明

### 前端状态

核心状态由 Zustand 管理：

- 当前作用域
- 环境变量列表
- 待应用变更
- 加载状态
- 最近错误
- 备份列表
- 写权限状态
- 最近操作与最近备份路径

### Tauri Commands

前端所有系统操作都通过 Tauri command 调用 Rust，不直接访问系统 API。主要 command 包括：

| Command | 说明 |
| --- | --- |
| `list_env_vars` | 列出指定作用域变量 |
| `get_env_var` | 获取单个变量 |
| `preview_changes` | 生成变更预览 |
| `apply_changes` | 权限检查、备份并应用变更 |
| `remove_env_var` | 删除变量 |
| `check_permission` | 检查写权限 |
| `create_backup` | 创建备份 |
| `list_backups` | 列出备份 |
| `preview_backup_restore` | 预览备份恢复差异 |
| `restore_backup` | 恢复备份 |
| `parse_path_var` | 拆分 Path 值 |
| `join_path_var` | 拼接 Path 值 |
| `restart_as_admin` | Windows 按需管理员重启 |

### 文件与目录选择

编辑变量时支持通过系统对话框选择目录或文件，功能由 Tauri dialog 插件提供。前端可以切换“目录 / 文件”模式，选择结果会填入当前输入框或列表条目。

### 语言

语言包位于 `src/locales/`：

- `zh-CN.json`
- `en.json`

语言选择保存在浏览器 `localStorage`，key 为 `varman.locale`。未设置时根据浏览器语言自动匹配，默认回退到简体中文。

## 测试与验证

```bash
# Rust 格式检查
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Rust 单元测试
cargo test --manifest-path src-tauri/Cargo.toml

# 前端类型检查与构建
pnpm build
```

建议在提交前依次运行以上三条命令。

## 打包

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：

| 平台 | 产物 |
| --- | --- |
| Windows | MSI、NSIS 安装包 |
| Linux | deb、AppImage |
| macOS | DMG |

## 持续集成

`.github/workflows/build.yml` 提供 Windows、Ubuntu 和 macOS 三平台构建，流程包括：

1. 安装 pnpm、Node.js 和 Rust。
2. 安装平台系统依赖。
3. 运行 Rust 格式检查和单元测试。
4. 构建前端。
5. 构建桌面安装包。
6. 上传构建产物。

macOS 签名与公证通过 GitHub Secrets 配置：

- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

### 自动发布

CI 在三平台构建和测试全部成功后，会自动创建 GitHub Release 并上传安装包：

- Windows：MSI、NSIS 安装包
- Linux：deb、AppImage
- macOS：DMG

发布由 `v*` 标签触发，例如：

```bash
git tag v0.1.0
git push origin v0.1.0
```

Release 任务会校验标签、`package.json` 和 `src-tauri/tauri.conf.json` 中的版本一致；不一致时发布失败。若同名 Release 已存在，会使用 `--clobber` 更新其资产，方便重跑修复发布。

## 常见问题

### 为什么修改变量后某些已运行程序没有生效？

系统环境变量的更新不会自动注入已经运行的进程。Windows 写入后 VarMan 会广播 `WM_SETTINGCHANGE`，资源管理器等监听该消息的程序可能自动更新，但多数应用仍需重启。Unix shell 配置需要重新打开 shell 或重新加载配置文件。

### Linux/macOS 如何修改系统变量？

以 root 身份启动 VarMan，例如：

```bash
sudo pnpm tauri dev
```

或在打包安装后使用系统提供的提权方式运行。

### 无权限时还能做什么？

VarMan 会进入只读模式，仍可查看、搜索和预览差异，但不能写入。Windows 会提供“以管理员身份重启”入口；Unix 会提示使用 `sudo` 或 `pkexec`。

### 为什么 Unix 会显示多个来源？

Unix 用户配置可能同时存在多个候选文件。VarMan 会解析存在的候选文件，并在来源列显示具体路径，避免用户误以为变量只来自一个文件。
