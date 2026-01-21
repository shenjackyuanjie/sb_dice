# sb_dice

一个用来解决 DICE 的 sb 字符串机制的字符串提取与替换工具。

## 简介

`sb_dice` 是一个基于 Rust 的 CLI 工具，用于解析 TypeScript 文件，将所有普通字符串字面量（不包括模板字符串的 quasis）替换为按顺序递增的索引字符串 `"0"`, `"1"`, ...，并生成对应的映射表。

## 功能特性

- ✅ 解析 TypeScript 文件并替换字符串字面量
- ✅ 生成替换后的 TS 文件（`<name>_r.ts`）
- ✅ 生成字符串映射表（`<name>_s.json`）
- ✅ 自动去除输出代码中的注释
- ✅ 完善的错误处理和友好的帮助信息

## 安装

### 前置要求

- Rust 2024 edition 或更高版本
- Cargo

### 构建

```bash
git clone <repository_url>
cd sb_dice
cargo build --release
```

构建完成后，可执行文件位于 `target/release/sb_dice.exe`（Windows）或 `target/release/sb_dice`（Linux/macOS）。

## 使用方法

### 基本用法

```bash
sb_dice <path/to/file.ts>
```

### 查看帮助

```bash
sb_dice -h
# 或
sb_dice --help
```

### 参数说明

- `<path/to/file.ts>`：输入的 TypeScript 文件路径
- `-h, --help`：显示帮助信息

## 输出说明

工具会生成两个文件：

1. **`<name>_r.ts`**：替换后的 TypeScript 文件，所有字符串字面量被替换为索引
2. **`<name>_s.json`**：映射表，格式如下：
   ```json
   {
     "0": "原始字符串0",
     "1": "原始字符串1",
     "2": "原始字符串2"
   }
   ```

## 示例

假设有一个 `example.ts` 文件：

```typescript
const name = "World";
console.log(`Hello, ${name}!`);
const greeting = "Welcome";
```

运行：

```bash
sb_dice example.ts
```

将生成：

**`example_r.ts`**：
```typescript
const name="0";
console.log(`Hello, ${name}!`);
const greeting="1";
```

**`example_s.json`**：
```json
{
  "0": "World",
  "1": "Welcome"
}
```

## 注意事项

- ⚠️ 不会替换模板字符串中的静态部分（quasis）
- ⚠️ 会替换 import/require 中的字符串模块路径
- ⚠️ 输出代码中会去除所有注释
- ⚠️ 仅支持 `.ts` 扩展名的文件作为输入

## 依赖

- `swc_core` ^55.0：TypeScript 解析和代码生成
- `serde_json` ^1.0：JSON 序列化

## 作者

shenjack & Gemini 3 Pro & GPT 5 mini & GLM 4.7 & DeepSeek v3.2（按照贡献多少排序🔥）

## 许可证

请查看项目根目录下的 LICENSE 文件了解详情。