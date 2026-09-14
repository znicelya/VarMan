# VarMan

VarMan 是一个基于 Tauri 2、React 19、TypeScript、Tailwind CSS 和 shadcn/ui 的跨平台环境变量可视化管理工具，支持 Windows、Linux 和 macOS。

## 功能

- 查看用户级和系统级环境变量，支持名称搜索和来源显示。
- 新增、编辑、删除环境变量，所有写入均先生成红绿 diff 预览。
- Path 变量逐条编辑，支持拖拽排序、缺失和重复提示、实时拼接预览。
- 写入前自动备份；Windows 备份到 `%APPDATA%\VarMan\backups`，Unix 备份到 `~/.VarMan/backups`。
- Windows 正确保留 `REG_EXPAND_SZ`，写入后广播 `WM_SETTINGCHANGE`。
- Unix 修改使用临时文件和原子替换，系统级写入要求 root 权限。
- 无权限时界面进入只读模式，Windows 支持按需以管理员身份重启。

## 开发

```bash
pnpm install
pnpm tauri dev
```

## 验证

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build
```

## 打包

```bash
pnpm tauri build
```

Windows 生成 MSI 和 NSIS 安装包；Linux 生成 deb 和 AppImage；macOS 生成 DMG。三平台构建由 `.github/workflows/build.yml` 覆盖。
