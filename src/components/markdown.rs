use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};

/// 创建 Markdown 预览组件，使用自定义样式
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    /// 渲染 Markdown 为 Ratatui Text，使用改进的样式
    pub fn render(markdown: &str) -> Text<'static> {
        let parsed = tui_markdown::from_str(markdown);
        let mut lines = Vec::new();
        let mut in_table = false;

        for line in parsed.lines {
            let mut spans = Vec::new();
            let line_content: String = line
                .spans
                .iter()
                .map(|s| &s.content[..])
                .collect::<Vec<_>>()
                .join("");

            // 检测表格行
            let is_table_row = line_content.starts_with('|') && line_content.ends_with('|');

            // 检测标题级别（通过检查第一个 span 内容）
            let heading_level = line
                .spans
                .first()
                .map(|span| {
                    if span.content.starts_with("# ") {
                        Some(1)
                    } else if span.content.starts_with("## ") {
                        Some(2)
                    } else if span.content.starts_with("### ") {
                        Some(3)
                    } else if span.content.starts_with("#### ") {
                        Some(4)
                    } else if span.content.starts_with("##### ") {
                        Some(5)
                    } else if span.content.starts_with("###### ") {
                        Some(6)
                    } else {
                        None
                    }
                })
                .flatten();

            for (idx, span) in line.spans.iter().enumerate() {
                let mut style = Self::convert_span_style(span.style);

                // 应用表格样式
                if is_table_row {
                    // 表头使用不同的颜色
                    style = style.fg(Color::LightGreen);
                    in_table = true;
                } else if in_table {
                    // 表格内容使用不同的颜色
                    style = style.fg(Color::White);
                }

                // 应用标题样式（只对非标记部分应用样式）
                if let Some(level) = heading_level {
                    if idx > 0 || !span.content.trim().starts_with('#') {
                        // 这是标题的实际内容，不是 # 标记
                        style = match level {
                            1 => style
                                .fg(Color::Yellow)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                            2 => style
                                .fg(Color::Green)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                            3 => style
                                .fg(Color::LightBlue)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                            4 => style
                                .fg(Color::LightMagenta)
                                .add_modifier(ratatui::style::Modifier::BOLD),
                            5 => style.fg(Color::LightYellow),
                            6 => style.fg(Color::LightCyan),
                            _ => style,
                        };
                    }
                }

                spans.push(Span::styled(span.content.to_string(), style));
            }

            // 如果不是表格行且之前在表格中，重置表格状态
            if !is_table_row && in_table {
                // 检查下一行是否是表格
                if !line_content.starts_with('|') {
                    in_table = false;
                }
            }

            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }

    /// 转换 span 样式，保留所有属性
    fn convert_span_style(core_style: ratatui_core::style::Style) -> Style {
        use ratatui_core::style::{Color as CoreColor, Modifier};

        let mut style = Style::default();

        // 转换前景色
        if let Some(fg) = core_style.fg {
            let color = match fg {
                CoreColor::Reset => Color::Reset,
                CoreColor::Black => Color::Black,
                CoreColor::Red => Color::Red,
                CoreColor::Green => Color::Green,
                CoreColor::Yellow => Color::Yellow,
                CoreColor::Blue => Color::Blue,
                CoreColor::Magenta => Color::Magenta,
                CoreColor::Cyan => Color::Cyan,
                CoreColor::Gray => Color::Gray,
                CoreColor::DarkGray => Color::DarkGray,
                CoreColor::LightRed => Color::LightRed,
                CoreColor::LightGreen => Color::LightGreen,
                CoreColor::LightYellow => Color::LightYellow,
                CoreColor::LightBlue => Color::LightBlue,
                CoreColor::LightMagenta => Color::LightMagenta,
                CoreColor::LightCyan => Color::LightCyan,
                CoreColor::White => Color::White,
                CoreColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
                CoreColor::Indexed(idx) => Color::Indexed(idx),
            };
            style = style.fg(color);
        }

        // 转换背景色（代码块会用到）
        if let Some(bg) = core_style.bg {
            let color = match bg {
                CoreColor::Reset => Color::Reset,
                CoreColor::Black => Color::Black,
                CoreColor::Red => Color::Red,
                CoreColor::Green => Color::Green,
                CoreColor::Yellow => Color::Yellow,
                CoreColor::Blue => Color::Blue,
                CoreColor::Magenta => Color::Magenta,
                CoreColor::Cyan => Color::Cyan,
                CoreColor::Gray => Color::Gray,
                CoreColor::DarkGray => Color::DarkGray,
                CoreColor::LightRed => Color::LightRed,
                CoreColor::LightGreen => Color::LightGreen,
                CoreColor::LightYellow => Color::LightYellow,
                CoreColor::LightBlue => Color::LightBlue,
                CoreColor::LightMagenta => Color::LightMagenta,
                CoreColor::LightCyan => Color::LightCyan,
                CoreColor::White => Color::White,
                CoreColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
                CoreColor::Indexed(idx) => Color::Indexed(idx),
            };
            style = style.bg(color);
        }

        // 转换修饰符
        let modifiers = core_style.add_modifier;
        if modifiers.contains(Modifier::BOLD) {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        if modifiers.contains(Modifier::ITALIC) {
            style = style.add_modifier(ratatui::style::Modifier::ITALIC);
        }
        if modifiers.contains(Modifier::UNDERLINED) {
            style = style.add_modifier(ratatui::style::Modifier::UNDERLINED);
        }

        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let markdown = r#"**Blod**, `block`

# This is h1
## This is h2
### This is h3
#### This is h4

[Link](https://example.com)"#;

        let text = MarkdownRenderer::render(markdown);

        println!("Rendered {} lines", text.lines.len());
        for (i, line) in text.lines.iter().enumerate() {
            println!("Line {}: {} spans", i, line.spans.len());
            for (j, span) in line.spans.iter().enumerate() {
                let style = &span.style;
                println!(
                    "  Span {}: '{}' fg={:?} bg={:?}",
                    j, span.content, style.fg, style.bg
                );
            }
        }
    }
}
