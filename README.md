# inR Remapper

Windows 键盘映射工具，基于 Tauri 2 + Rust 开发。
这是本人被steam古老游戏不支持按键映射切换不得已开发的运行时键盘映射修改程序
点名批评 终末地的空格冲刺，还不能换成跳跃
国王 两个王冠的抽象双人按键
红警 2 3代快捷键差异极大
## 功能

- 键盘按键映射（A → B）
- 支持禁用按键（映射到"无"）
- 多配置文件管理
- 系统托盘运行
- 数据持久化存储
## 支持的按键

| 类型 | 按键 |
|------|------|
| 字母 | A-Z |
| 数字 | 0-9 |
| 功能键 | F1-F12 |
| 控制键 | Space, Enter, Tab, Escape |
| 方向键 | ←↑→↓ |
| 特殊 | 禁用（无输出） |

## 使用方法

1. 运行程序
2. 选择或创建配置文件
3. 选择原按键和目标按键，点击"添加映射"
4. 映射自动生效
5. 关闭窗口自动最小化到托盘
6. 右键托盘图标可暂停/恢复或退出

## 技术栈

- **后端**: Rust + Windows API (WH_KEYBOARD_LL)
- **前端**: HTML/CSS/JavaScript
- **框架**: Tauri 2
- **数据库**: SQLite

## 数据存储

配置数据保存在:
```
%APPDATA%\inR_Remapper\config.db
```

## 开发

```bash
# 安装依赖
npm install

# 开发模式
cd src-tauri && cargo tauri dev

# 构建
npm run build
cd src-tauri && cargo tauri build
```

## 注意事项

- 映射链式反应已被防止（A→B, B→S 不会导致 A→S）
- 模拟的按键不会再次被映射
- 程序关闭后自动最小化到托盘，不退出
