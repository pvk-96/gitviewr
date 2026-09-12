//! A minimal, dependency-free PDF generator producing an A4 text report using
//! the PDF standard base-14 Helvetica fonts (no font embedding required).

use crate::git::errors::{AnalysisError, Result};
use crate::models::analysis::AnalysisData;
use crate::utils::dates;

const PAGE_W: f32 = 595.28; // A4 width in points
const PAGE_H: f32 = 841.89; // A4 height in points
const MARGIN_X: f32 = 50.0;
const MARGIN_TOP: f32 = 50.0;
const MARGIN_BOTTOM: f32 = 50.0;
const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN_X;
const FONT_SIZE: f32 = 9.0;
const FONT_SIZE_HEADER: f32 = 16.0;
const FONT_SIZE_SUBHEADER: f32 = 11.0;

pub fn write(data: &AnalysisData, path: &str) -> Result<()> {
    let mut pdf = PdfWriter::new();
    pdf.font("F1", "Helvetica");
    pdf.font("F2", "Helvetica-Bold");
    for line in build_text(data) {
        match line {
            Line::Heading(text) => pdf.line(&text, "F2", FONT_SIZE_HEADER),
            Line::SubHeading(text) => pdf.line(&text, "F2", FONT_SIZE_SUBHEADER),
            Line::Normal(text) => pdf.line(&text, "F1", FONT_SIZE),
        }
    }
    std::fs::write(path, pdf.finish())
        .map_err(|e| AnalysisError::ExportFailed(format!("Could not write the PDF file: {e}")))?;
    Ok(())
}

enum Line {
    Heading(String),
    SubHeading(String),
    Normal(String),
}

fn build_text(data: &AnalysisData) -> Vec<Line> {
    let mut lines = Vec::new();

    lines.push(Line::Heading("GitViewr Report".into()));
    lines.push(Line::Normal(format!(
        "Repository: {}",
        data.repository.name
    )));
    lines.push(Line::Normal(format!("Path: {}", data.metadata.path)));
    lines.push(Line::Normal(format!("Generated: {}", dates::now_iso())));
    lines.push(Line::Normal(String::new()));

    lines.push(Line::SubHeading("Statistics".into()));
    lines.push(Line::Normal(format!(
        "Branch: {}",
        data.stats.branch.as_deref().unwrap_or("-")
    )));
    lines.push(Line::Normal(format!(
        "Commits: {}",
        data.stats.commit_count
    )));
    lines.push(Line::Normal(format!(
        "Branches: {}",
        data.stats.branch_count
    )));
    lines.push(Line::Normal(format!(
        "Tracked Files: {}",
        data.stats.tracked_files
    )));
    lines.push(Line::Normal(format!(
        "Contributors: {}",
        data.stats.contributor_count
    )));
    lines.push(Line::Normal(format!(
        "Total Additions: +{}",
        data.stats.total_additions
    )));
    lines.push(Line::Normal(format!(
        "Total Deletions: -{}",
        data.stats.total_deletions
    )));
    lines.push(Line::Normal(format!(
        "Net Change: {}",
        data.stats.net_change
    )));
    lines.push(Line::Normal(String::new()));

    if !data.analytics.contributors.is_empty() {
        lines.push(Line::SubHeading("Contributors".into()));
        for c in &data.analytics.contributors {
            lines.push(Line::Normal(format!(
                "  {} - {} commits (+{}/-{})",
                c.name, c.commits, c.additions, c.deletions
            )));
        }
        lines.push(Line::Normal(String::new()));
    }

    if !data.analytics.commits_over_time.is_empty() {
        lines.push(Line::SubHeading("Commits Over Time".into()));
        for b in &data.analytics.commits_over_time {
            lines.push(Line::Normal(format!(
                "  {} - {} commits (+{}/-{})",
                b.period, b.commits, b.additions, b.deletions
            )));
        }
        lines.push(Line::Normal(String::new()));
    }

    let hotspots: Vec<_> = data.analytics.file_hotspots.iter().take(15).collect();
    if !hotspots.is_empty() {
        lines.push(Line::SubHeading("File Hotspots (Top 15)".into()));
        for h in hotspots {
            lines.push(Line::Normal(format!(
                "  {} - {} changes (+{}/-{})",
                h.path, h.changes, h.additions, h.deletions
            )));
        }
        lines.push(Line::Normal(String::new()));
    }

    if !data.commits.is_empty() {
        lines.push(Line::SubHeading(format!(
            "Commits ({})",
            data.commits.len()
        )));
        for c in data.commits.iter().take(200) {
            let hash = if c.hash.len() > 7 {
                &c.hash[..7]
            } else {
                &c.hash
            };
            lines.push(Line::Normal(format!(
                "  {}  {}  {:<18}  {} (+{}/-{})",
                hash,
                &c.author_date[..10.min(c.author_date.len())],
                truncate(&c.author_name, 18),
                truncate(&c.subject, 60),
                c.additions,
                c.deletions
            )));
        }
        if data.commits.len() > 200 {
            lines.push(Line::Normal(format!(
                "  ... and {} more commits.",
                data.commits.len() - 200
            )));
        }
    }

    lines
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let chars: Vec<char> = s.chars().collect();
        format!(
            "{}...",
            chars[..max.saturating_sub(3)].iter().collect::<String>()
        )
    }
}

// ──────────────────────────────────────────────────────────────────────────

struct PdfFont {
    tag: String,
    base: String,
}

struct PdfLine {
    content: String,
    font_tag: String,
    size: f32,
    height: f32,
}

struct PdfPage {
    lines: Vec<PdfLine>,
    y: f32,
}

struct PdfWriter {
    fonts: Vec<PdfFont>,
    pages: Vec<PdfPage>,
    next_font_num: usize,
}

impl PdfWriter {
    fn new() -> Self {
        PdfWriter {
            fonts: Vec::new(),
            pages: Vec::new(),
            next_font_num: 1,
        }
    }

    fn font(&mut self, tag: &str, base: &str) {
        if !self.fonts.iter().any(|f| f.tag == tag) {
            self.fonts.push(PdfFont {
                tag: tag.to_string(),
                base: base.to_string(),
            });
            self.next_font_num += 1;
        }
    }

    fn line(&mut self, content: &str, font_tag: &str, size: f32) {
        let height = size * 1.55;
        for wrapped in wrap(content, size) {
            self.emit(wrapped, font_tag, size, height);
        }
    }

    fn emit(&mut self, content: String, font_tag: &str, size: f32, height: f32) {
        if self.pages.is_empty() || self.pages.last().unwrap().y - height < MARGIN_BOTTOM {
            self.pages.push(PdfPage {
                lines: Vec::new(),
                y: PAGE_H - MARGIN_TOP,
            });
        }
        let page = self.pages.last_mut().unwrap();
        page.y -= height;
        page.lines.push(PdfLine {
            content,
            font_tag: font_tag.to_string(),
            size,
            height,
        });
    }

    fn finish(self) -> Vec<u8> {
        let page_count = self.pages.len();
        let font_count = self.fonts.len();

        // Object layout:
        //   1: Catalog
        //   2: Pages
        //   3 .. 3+font_count-1: fonts
        //   fs .. fs+page_count-1: content streams
        //   cs .. cs+page_count-1: page objects
        let font_start = 3usize;
        let content_start = font_start + font_count;
        let page_start = content_start + page_count;
        let total_objects = page_start + page_count; // highest object number

        let mut out = Vec::with_capacity(128 * 1024);
        let mut xref: Vec<usize> = Vec::new();
        let push_obj = |out: &mut Vec<u8>, xref: &mut Vec<usize>, value: Vec<u8>| {
            xref.push(out.len());
            out.extend_from_slice(&value);
            out.extend_from_slice(b"\n");
        };

        out.extend_from_slice(b"%PDF-1.4\n");

        // 1: Catalog
        push_obj(
            &mut out,
            &mut xref,
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj".to_vec(),
        );

        // 2: Pages tree
        let kids: Vec<String> = (0..page_count)
            .map(|i| format!("{} 0 R", page_start + i))
            .collect();
        let pages = format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj",
            kids.join(" "),
            page_count
        );
        push_obj(&mut out, &mut xref, pages.into_bytes());

        // Fonts
        for (i, font) in self.fonts.iter().enumerate() {
            let obj = font_start + i;
            let entry = format!(
                "{obj} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /{} >>\nendobj",
                font.base
            );
            push_obj(&mut out, &mut xref, entry.into_bytes());
        }

        // Content streams + pages
        for (i, page) in self.pages.iter().enumerate() {
            let content_obj = content_start + i;
            let page_obj = page_start + i;

            let mut stream: Vec<u8> = Vec::new();
            let mut y = PAGE_H - MARGIN_TOP;
            for line in &page.lines {
                let encoded = pdf_string(&line.content);
                stream.extend_from_slice(
                    format!(
                        "BT /{} {} Tf {} {} Td ({}) Tj ET\n",
                        line.font_tag, line.size, MARGIN_X, y, encoded
                    )
                    .as_bytes(),
                );
                y -= line.height;
            }

            let content_entry = format!(
                "{content_obj} 0 obj\n<< /Length {} >>\nstream\n",
                stream.len()
            );
            let mut content_value = content_entry.into_bytes();
            content_value.extend_from_slice(&stream);
            content_value.extend_from_slice(b"\nendstream\nendobj");
            push_obj(&mut out, &mut xref, content_value);

            let fonts_refs = self
                .fonts
                .iter()
                .enumerate()
                .map(|(i, f)| format!("/{} {} 0 R", f.tag, font_start + i))
                .collect::<Vec<_>>()
                .join(" ");
            let page_entry = format!(
                "{page_obj} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W:.2} {PAGE_H:.2}] /Contents {content_obj} 0 R /Resources << /Font << {fonts_refs} >> >> >>\nendobj"
            );
            push_obj(&mut out, &mut xref, page_entry.into_bytes());
        }

        // Cross-reference
        let xref_offset = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", total_objects + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for off in &xref {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        // Pad if xref_entries were shorter than total_objects (shouldn't happen)
        while xref.len() < total_objects {
            out.extend_from_slice(b"0000000000 00000 f \n");
            xref.push(usize::MAX);
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                total_objects + 1
            )
            .as_bytes(),
        );

        out
    }
}

/// Wrap text into segments that fit the content width at the given font size.
fn wrap(text: &str, size: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    // Approximate Helvetica average glyph width at ~0.5 * fontSize.
    let char_w = size * 0.5;
    let max_chars = (CONTENT_W / char_w).floor() as usize;
    if max_chars == 0 {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if current.chars().count() >= max_chars {
            out.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    out.push(current);
    out
}

/// Escape and approximate-Latin-1 encode a string for PDF string literals.
fn pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        if c <= '\u{FF}' {
            match c {
                '\\' => out.push_str("\\\\"),
                '(' => out.push_str("\\("),
                ')' => out.push_str("\\)"),
                '\n' | '\r' => {}
                '\t' => out.push(' '),
                _ => out.push(c),
            }
        } else {
            // Non-Latin-1 characters are not representable in the base-14 fonts.
            out.push('?');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_long_lines() {
        let text = "a".repeat(300);
        let chunks = wrap(&text, FONT_SIZE);
        assert!(chunks.len() > 1, "line should be wrapped");
        let max_chars = (CONTENT_W / (FONT_SIZE * 0.5)).floor() as usize;
        assert!(chunks.iter().all(|c| c.chars().count() <= max_chars));
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn escapes_specials() {
        assert_eq!(pdf_string("a(b)\\c"), "a\\(b\\)\\\\c");
        assert_eq!(pdf_string("café"), "café");
        assert_eq!(pdf_string("雪"), "?");
    }

    #[test]
    fn writer_produces_pdf_header() {
        let mut pdf = PdfWriter::new();
        pdf.font("F1", "Helvetica");
        pdf.line("hello", "F1", FONT_SIZE);
        let out = pdf.finish();
        assert!(out.starts_with(b"%PDF-"));
        assert!(out.windows(5).any(|w| w == b"%%EOF"));
        assert!(out.windows(5).any(|w| w == b"xref\n"));
    }
}
