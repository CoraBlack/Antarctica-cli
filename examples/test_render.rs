use ratatui::{
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

fn main() {
    // Simulated blog content
    let blog_content = r#"# What is antarctica

This is a **bold** text and this is *italic*.

## Features

- Feature 1
- Feature 2
"#;

    println!("Testing markdown rendering in simulated TUI...\n");
    println!("Blog content:");
    println!("{}", blog_content);
    println!("\n{}", "=".repeat(60));

    // Parse the markdown
    let markdown_text = tui_markdown::from_str(blog_content);

    println!("\nParsed markdown lines: {}", markdown_text.lines.len());

    // Simulate the rendering like in blog_view.rs
    let area = Rect::new(0, 0, 80, 20);
    let block = Block::default()
        .title("渲染模式")
        .borders(Borders::ALL)
        .border_style(Color::Cyan);

    let inner_area = block.inner(area);
    println!("Area: {:?}", area);
    println!("Inner area: {:?}", inner_area);

    // Try to render using the simpler approach - just use the Text directly
    println!("\n{}", "=".repeat(60));
    println!("\nAttempting to render manually (like blog_view.rs)...");

    let visible_lines = inner_area.height as usize;
    let line_count = markdown_text.lines.len();

    println!(
        "Total lines: {}, Visible lines: {}",
        line_count, visible_lines
    );

    // Show first few lines
    for (i, line) in markdown_text.lines.iter().take(visible_lines).enumerate() {
        print!("Line {}: ", i);
        for span in &line.spans {
            let style_desc = if span
                .style
                .add_modifier
                .contains(ratatui_core::style::Modifier::BOLD)
            {
                "[BOLD]"
            } else if span
                .style
                .add_modifier
                .contains(ratatui_core::style::Modifier::ITALIC)
            {
                "[ITALIC]"
            } else {
                ""
            };
            print!("{}{} ", span.content, style_desc);
        }
        println!();
    }

    // Now let's check if using Paragraph with the Text works
    println!("\n{}", "=".repeat(60));
    println!("\nTesting with TestBackend...");

    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            let area = frame.area();

            // Render block
            frame.render_widget(block, area);

            // Get inner area
            let inner_area = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

            // Try rendering the markdown text
            // Note: We need to use a widget that can render ratatui_core::text::Text
            // For now, let's manually render like the original code does
            for (i, line) in markdown_text
                .lines
                .iter()
                .take(inner_area.height as usize)
                .enumerate()
            {
                let y = inner_area.y + i as u16;

                let mut x = inner_area.x;
                for span in &line.spans {
                    let content = &span.content;
                    let remaining_width = (inner_area.x + inner_area.width).saturating_sub(x);
                    if remaining_width == 0 {
                        break;
                    }

                    // Calculate width
                    let mut display_content = String::new();
                    let mut current_width = 0u16;
                    for c in content.chars() {
                        let char_width = if c.is_ascii() { 1 } else { 2 };
                        if current_width + char_width as u16 > remaining_width {
                            break;
                        }
                        display_content.push(c);
                        current_width += char_width as u16;
                    }

                    if !display_content.is_empty() {
                        // Convert style
                        let style = convert_style(span.style);
                        frame.render_widget(
                            Paragraph::new(display_content).style(style),
                            Rect::new(x, y, current_width, 1),
                        );
                        x += current_width;
                    }
                }
            }
        })
        .unwrap();

    // Get the buffer content
    let buffer = terminal.backend().buffer().clone();

    println!("\nRendered output (first 10 lines):");
    for y in 0..10 {
        let mut line = String::new();
        for x in 0..80 {
            let cell = buffer.get(x, y);
            line.push(cell.symbol().chars().next().unwrap_or(' '));
        }
        println!("{}", line);
    }

    println!("\n{}", "=".repeat(60));
    println!("✓ Rendering test completed!");
    println!("\nDebug tips:");
    println!("1. Check if blog.content is not empty");
    println!("2. Verify the area dimensions are correct");
    println!("3. Ensure scroll_offset is 0 initially");
    println!("4. The manual span rendering should preserve styling");
}

fn convert_style(core_style: ratatui_core::style::Style) -> Style {
    use ratatui_core::style::{Color as CoreColor, Modifier};

    let mut style = Style::default();

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
