# LiteMD 演示文档

[TOC]

> LiteMD 是一个只做阅读的轻量 Markdown 阅读器。本文档用于演示渲染功能。

## 一、基础语法

支持 **粗体**、*斜体*、~~删除线~~、`行内代码`、[链接](https://example.com)。

### 1.1 列表

- 无序列表项
  - 嵌套项
- 另一项

1. 有序项一
2. 有序项二

### 1.2 任务列表

- [x] 已完成的阅读任务
- [ ] 待办事项
- [ ] 另一项

## 二、表格

| 功能 | 优先级 | 说明 |
|------|--------|------|
| Markdown 渲染 | P0 | GFM 全语法 |
| 阅读进度记忆 | P0 | 精确到像素 |
| 目录大纲 | P0 | 树形 + 高亮 |
| 明暗主题 | P1 | 跟随系统 |

## 三、代码块

### 3.1 Java 示例

```java
public class HelloWorld {
    public static void main(String[] args) {
        // 你好，世界
        for (int i = 0; i < 3; i++) {
            System.out.println("Hello " + i);
        }
    }
}
```

### 3.2 JavaScript 示例

```javascript
const reader = { name: 'LiteMD', light: true }
reader.highlight = (code) => code.replace(/console/g, 'log')
console.log(reader)
```

### 3.3 Shell 示例

```bash
npm run tauri dev     # 开发运行
npm run tauri build   # 打包安装
```

## 四、引用与分割线

> 引用块：阅读进度是第一优先级功能。
>
> 精确到像素级滚动位置，关掉再打开，还在上次读到的地方。

---

分隔线上方是引用。

## 五、脚注与公式（纯文本回退）

数学公式 `$E = mc^2$` 以行内代码方式显示（未启用 KaTeX，保持轻量）。

## 六、图片说明

图片支持本地路径（相对 Markdown 文件目录）与点击放大。

## 七、长文档测试段落

以下为填充段落，用于测试长文档渲染性能与阅读进度记忆。打开本文档后滚动到中部，关闭再重新打开，应精确恢复到上次位置。

Lorem ipsum dolor sit amet, consectetur adipiscing elit. 中文测试文字，用于验证中文字体渲染效果。Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

## 八、结束语

感谢使用 LiteMD。祝你阅读愉快！
