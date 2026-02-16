//! Markdown processing utilities with syntax highlighting.

use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Markdown renderer with syntax highlighting.
pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownRenderer {
    /// Create a new markdown renderer.
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// Render markdown to HTML with syntax highlighting.
    pub fn render(&self, markdown: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        let parser = Parser::new_ext(markdown, options);
        let parser = SyntaxHighlighter::new(parser, &self.syntax_set, &self.theme_set);

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }

    /// Calculate reading time in minutes.
    pub fn reading_time(content: &str) -> i32 {
        let word_count = content.split_whitespace().count();
        let minutes = (word_count as f64 / 200.0).ceil() as i32;
        minutes.max(1)
    }

    /// Extract excerpt from content (first paragraph or N characters).
    pub fn extract_excerpt(content: &str, max_chars: usize) -> String {
        // Try to get the first paragraph
        let first_para = content
            .split("\n\n")
            .find(|p| !p.trim().is_empty() && !p.starts_with('#'))
            .unwrap_or(content);

        // Strip markdown syntax roughly
        let stripped: String = first_para
            .chars()
            .filter(|c| !['#', '*', '_', '`', '[', ']', '(', ')'].contains(c))
            .collect();

        if stripped.len() <= max_chars {
            stripped.trim().to_string()
        } else {
            let truncated: String = stripped.chars().take(max_chars).collect();
            // Try to break at a word boundary
            if let Some(last_space) = truncated.rfind(' ') {
                format!("{}...", &truncated[..last_space])
            } else {
                format!("{}...", truncated)
            }
        }
    }
}

/// Iterator adapter for syntax highlighting code blocks.
struct SyntaxHighlighter<'a, I: Iterator<Item = Event<'a>>> {
    inner: I,
    syntax_set: &'a SyntaxSet,
    theme_set: &'a ThemeSet,
    in_code_block: bool,
    code_lang: Option<String>,
    code_buffer: String,
}

impl<'a, I: Iterator<Item = Event<'a>>> SyntaxHighlighter<'a, I> {
    fn new(inner: I, syntax_set: &'a SyntaxSet, theme_set: &'a ThemeSet) -> Self {
        Self {
            inner,
            syntax_set,
            theme_set,
            in_code_block: false,
            code_lang: None,
            code_buffer: String::new(),
        }
    }

    fn highlight_code(&self, code: &str, lang: &str) -> String {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Use a theme that works well in both light and dark modes
        let theme = &self.theme_set.themes["InspiredGitHub"];

        match highlighted_html_for_string(code, self.syntax_set, syntax, theme) {
            Ok(html) => {
                // Wrap in a container with the language class
                format!(
                    r#"<div class="code-block" data-lang="{}"><pre><code class="language-{}">{}</code></pre></div>"#,
                    lang, lang, html
                )
            }
            Err(_) => {
                // Fallback to plain code block
                format!(
                    r#"<pre><code class="language-{}">{}</code></pre>"#,
                    lang,
                    html_escape(code)
                )
            }
        }
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for SyntaxHighlighter<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let event = self.inner.next()?;

            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    self.in_code_block = true;
                    self.code_buffer.clear();
                    self.code_lang = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            let lang_str = lang.to_string();
                            if lang_str.is_empty() {
                                Some("text".to_string())
                            } else {
                                Some(lang_str)
                            }
                        }
                        CodeBlockKind::Indented => Some("text".to_string()),
                    };
                    continue;
                }
                Event::End(TagEnd::CodeBlock) => {
                    self.in_code_block = false;
                    let lang = self.code_lang.take().unwrap_or_else(|| "text".to_string());
                    let highlighted = self.highlight_code(&self.code_buffer, &lang);
                    self.code_buffer.clear();
                    return Some(Event::Html(highlighted.into()));
                }
                Event::Text(text) if self.in_code_block => {
                    self.code_buffer.push_str(&text);
                    continue;
                }
                _ => return Some(event),
            }
        }
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_time() {
        let short = "Hello world";
        assert_eq!(MarkdownRenderer::reading_time(short), 1);

        let medium = "word ".repeat(400);
        assert_eq!(MarkdownRenderer::reading_time(&medium), 2);
    }

    #[test]
    fn test_extract_excerpt() {
        let content = "# Title\n\nThis is the first paragraph.\n\nSecond paragraph.";
        let excerpt = MarkdownRenderer::extract_excerpt(content, 100);
        assert!(excerpt.contains("first paragraph"));
    }
}
