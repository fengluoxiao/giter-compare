# Giter Compare

一个基于 Tauri + Vue 3 开发的 Git 文件对比工具，类似于 Beyond Compare，提供直观的文件差异查看体验。

## ✨ 亮点

- **行级对比** - 精确到行的差异展示，支持新增、删除、修改的清晰标识
- **实时同步** - 文件变更自动检测并刷新，无需手动操作
- **双栏视图** - 左右分栏对比，支持同步滚动
- **树形目录** - 清晰的文件树结构，支持展开/折叠目录
- **多模式查看** - 支持工作区和暂存区切换查看
- **浅色/深色主题** - 支持主题切换，适应不同环境
- **文件过滤** - 可选择显示所有文件或仅显示变更文件
- **二进制文件检测** - 自动识别二进制文件并显示占位符
- **文本对比** - 支持直接粘贴文本进行对比
- **全局搜索** - 支持在整个项目中搜索代码，支持正则表达式和通配符
- **行内搜索** - 在差异对比中快速搜索代码内容

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) (推荐 v18+)
- [Rust](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/) (可选，但推荐安装)

### 安装依赖

```bash
npm install
```

### 开发模式运行

```bash
# 方式1：使用 Tauri CLI
npm run tauri dev

# 方式2：分别运行前端和后端
# 终端1：运行前端开发服务器
npm run dev

# 终端2：运行 Tauri 应用
cargo tauri dev
```

开发模式将启动：
- Vite 开发服务器 (默认 http://localhost:1420)
- Tauri 桌面应用窗口
- 支持热重载 (Hot Module Replacement)

### 构建生产版本

```bash
# 构建当前平台的安装包
npm run tauri build

# 构建特定平台
# macOS
npm run tauri build -- --target aarch64-apple-darwin
npm run tauri build -- --target x86_64-apple-darwin

# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

构建输出位置：
- **macOS**: `src-tauri/target/release/bundle/dmg/*.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/*.msi`
- **Linux**: `src-tauri/target/release/bundle/appimage/*.AppImage`

## 📖 使用说明

### 基本操作

1. **打开仓库** - 点击"打开文件夹"选择 Git 仓库
2. **查看变更** - 左侧面板显示文件变更列表
3. **对比文件** - 点击文件在右侧面板查看差异
4. **切换视图** - 使用"工作区"/"暂存区"标签切换查看模式
5. **主题切换** - 点击主题按钮切换浅色/深色模式
6. **导航** - 使用"上一个"/"下一个"按钮快速切换文件

### 全局搜索

- **快捷键**: `Ctrl+Alt+F` (Windows/Linux) / `Cmd+Control+F` (macOS)
- **功能**: 在整个项目中搜索代码内容
- **支持**: 正则表达式模式 (点击 `.*` 按钮切换)
- **文件过滤**: 支持按文件类型过滤搜索结果
- **结果展示**: 显示文件名、行号和匹配内容
- **代码预览**: 点击搜索结果可在弹窗中查看代码差异对比
- **跳转**: 点击"在主窗口打开"可跳转到主窗口查看完整差异

### 行内搜索

- **快捷键**: `Ctrl+F` (Windows/Linux) / `Cmd+F` (macOS)
- **功能**: 在当前差异对比中搜索代码
- **支持**: 通配符搜索 (如 `l*ke` 匹配 `like`)
- **导航**: 支持上一个/下一个匹配项跳转

## 🛠️ 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri
- **Git 操作**: git2-rs (libgit2 绑定)
- **文件监控**: notify (Rust 文件系统监控库)
- **UI 组件**: 自定义组件

## 📄 许可证

本项目采用 [GNU Affero General Public License v3.0](LICENSE) (AGPL-3.0) 开源协议。

Copyright (C) 2026 fengluoxiao

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📧 联系方式

如有问题或建议，请通过 GitHub Issues 联系。
