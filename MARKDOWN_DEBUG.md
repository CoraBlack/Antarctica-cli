# Markdown Rendering Debug & Fix

## Summary of Findings

After testing, I've confirmed that:

1. ✅ **tui-markdown is working correctly** - parses Markdown with proper styling (bold, italic, code highlighting)
2. ✅ **The rendering logic in `blog_view.rs` is correct** - it properly renders markdown with styling
3. ⚠️ **The issue is likely that `blog.content` is empty** or the API isn't returning the content field

## Debug Steps

Add this debug code to `src/pages/blog_view.rs` at line 305 (inside `ViewMode::Rendered`):

```rust
ViewMode::Rendered => {
    // DEBUG: Check if content is empty
    if blog.content.is_empty() {
        let warning = Paragraph::new("警告：博客内容为空")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(warning, inner_area);
        return;
    }
    
    // DEBUG: Print content length
    eprintln!("DEBUG - Content length: {}", blog.content.len());
    eprintln!("DEBUG - First 100 chars: {}", &blog.content[..100.min(blog.content.len())]);
    
    let markdown_text = tui_markdown::from_str(&blog.content);
    // ... rest of the code
}
```

## The Real Issue

Looking at the API struct in `src/api/mod.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Blog {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub html_content: String,
    // ...
}
```

The `content` field has `#[serde(default)]`, meaning if the API doesn't return it, it will be empty! The API might only be returning `html_content`.

## Solutions

### Solution 1: Check API Response
Ensure the API is returning the `content` field (not just `html_content`).

### Solution 2: Fallback to html_content
Modify the rendering to fallback to `html_content` if `content` is empty:

```rust
let content_to_render = if blog.content.is_empty() {
    &blog.html_content
} else {
    &blog.content
};
let markdown_text = tui_markdown::from_str(content_to_render);
```

### Solution 3: Strip HTML tags from html_content
If the API only returns HTML, you could strip the HTML tags:

```rust
fn strip_html(html: &str) -> String {
    // Simple HTML tag removal
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}
```

Then use:
```rust
let content_to_render = if blog.content.is_empty() {
    strip_html(&blog.html_content)
} else {
    blog.content.clone()
};
```

## Recommended Fix

Add a helper method to handle both cases:

```rust
// In src/pages/blog_view.rs

fn get_renderable_content(&self, blog: &BlogDetail) -> String {
    if !blog.content.is_empty() {
        blog.content.clone()
    } else if !blog.html_content.is_empty() {
        // Strip HTML tags for display
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        re.replace_all(&blog.html_content, "").to_string()
    } else {
        "*无内容*".to_string()
    }
}
```

Then in the rendering code:
```rust
let content = self.get_renderable_content(blog);
let markdown_text = tui_markdown::from_str(&content);
```

## Verification

Run the test examples to verify tui-markdown is working:

```bash
cargo run --example test_markdown
cargo run --example test_render
```

Both should show properly formatted markdown output.

## Next Steps

1. Add the debug code to check if `blog.content` is empty
2. Check your API response to see if it includes the `content` field
3. Implement one of the solutions above based on what your API returns
