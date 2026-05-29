use crate::doc::{
    chars_to_string, group_runs, Block, Doc, Image, PdfBox, PdfElement, PdfPage, PdfTextFragment,
    StyledChar, Table,
};

pub fn render_editor_preview_html(doc: &Doc) -> String {
    let mut out = String::new();
    out.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("  <meta charset=\"utf-8\">\n");
    out.push_str("  <title>Editor rich_text preview</title>\n");
    out.push_str("  <style>");
    out.push_str(editor_preview_css());
    out.push_str("</style>\n</head>\n<body>\n");
    out.push_str("  <main class=\"rich-text-preview\">\n");
    for block in &doc.blocks {
        render_block(&mut out, block);
    }
    out.push_str("  </main>\n</body>\n</html>\n");
    out
}

fn render_block(out: &mut String, block: &Block) {
    match block {
        Block::PdfPage(page) => render_pdf_page(out, page),
        Block::Heading(level, runs) => {
            let level = (*level).clamp(1, 6);
            out.push_str(&format!("    <h{level}>"));
            render_runs(out, runs);
            out.push_str(&format!("</h{level}>\n"));
        }
        Block::Paragraph(runs) => {
            out.push_str("    <p>");
            render_runs(out, runs);
            out.push_str("</p>\n");
        }
        Block::Blockquote(runs) => {
            out.push_str("    <blockquote>");
            render_runs(out, runs);
            out.push_str("</blockquote>\n");
        }
        Block::Bullet(runs) => {
            out.push_str("    <p class=\"rich-text-list-item\"><span>* </span>");
            render_runs(out, runs);
            out.push_str("</p>\n");
        }
        Block::Numbered(runs) => {
            out.push_str("    <p class=\"rich-text-list-item\"><span>1. </span>");
            render_runs(out, runs);
            out.push_str("</p>\n");
        }
        Block::Pre(runs) => {
            out.push_str("    <pre>");
            out.push_str(&escape_html(&chars_to_string(runs)));
            out.push_str("</pre>\n");
        }
        Block::Table(table) => render_table(out, table),
        Block::Image(image) => render_image(out, image),
        Block::PageBreak(page) => {
            let label = page
                .map(|page| format!("--- page {page} ---"))
                .unwrap_or_else(|| "--- page break ---".into());
            out.push_str("    <p class=\"rich-text-note\">");
            out.push_str(&escape_html(&label));
            out.push_str("</p>\n");
        }
        Block::PagePlaceholder { page, reason } => {
            let page = page
                .map(|page| page.to_string())
                .unwrap_or_else(|| "?".into());
            out.push_str("    <p class=\"rich-text-note\">");
            out.push_str(&escape_html(&format!("[page {page}: {reason}]")));
            out.push_str("</p>\n");
        }
        Block::RawHtml(html) => {
            out.push_str("    <pre class=\"rich-text-raw\">");
            out.push_str(&escape_html(html));
            out.push_str("</pre>\n");
        }
        Block::Hr => out.push_str("    <hr>\n"),
    }
}

fn render_pdf_page(out: &mut String, page: &PdfPage) {
    let width = page.width_pt.unwrap_or_else(|| inferred_page_width(page));
    let height = page.height_pt.unwrap_or_else(|| inferred_page_height(page));
    out.push_str("    <section class=\"rich-text-pdf-page\"");
    if let Some(page_number) = page.page {
        out.push_str(&format!(" data-page=\"{page_number}\""));
    }
    out.push_str(&format!(
        " style=\"width:{:.2}pt;height:{:.2}pt\">\n",
        width.max(1.0),
        height.max(1.0)
    ));
    for element in &page.elements {
        render_pdf_element(out, element);
    }
    out.push_str("    </section>\n");
}

fn render_pdf_element(out: &mut String, element: &PdfElement) {
    match element {
        PdfElement::Shape(shape) => {
            let mut style = bounds_style(&shape.bounds);
            if let Some(background) = &shape.background {
                push_style(&mut style, "background", background);
            } else if shape.border.is_none() {
                push_style(&mut style, "background", "#d3d3d3");
            }
            if let Some(border) = &shape.border {
                push_style(&mut style, "border", border);
            }
            out.push_str(&format!(
                "      <div class=\"rich-text-pdf-shape\" style=\"{}\"></div>\n",
                escape_attr(&style)
            ));
        }
        PdfElement::Image(image) => {
            let style = bounds_style(&image.bounds);
            let label = if image.alt.is_empty() {
                "image"
            } else {
                image.alt.as_str()
            };
            out.push_str(&format!(
                "      <div class=\"rich-text-pdf-image\" style=\"{}\">{}</div>\n",
                escape_attr(&style),
                escape_html(label)
            ));
        }
        PdfElement::Text(text) => render_pdf_text(out, text),
        PdfElement::Ink(ink) => {
            out.push_str(&format!(
                "      <div class=\"rich-text-pdf-ink\" style=\"{}\"></div>\n",
                escape_attr(&bounds_style(&ink.bounds))
            ));
        }
        PdfElement::Link(link) => {
            out.push_str(&format!(
                "      <a class=\"rich-text-pdf-link\" href=\"{}\" style=\"{}\"></a>\n",
                escape_attr(&link.href),
                escape_attr(&bounds_style(&link.bounds))
            ));
        }
    }
}

fn render_pdf_text(out: &mut String, text: &PdfTextFragment) {
    let mut style = String::new();
    if let Some(left) = text.bounds.left_pt {
        push_style(&mut style, "left", &format!("{left:.2}pt"));
    }
    if let Some(top) = text.bounds.top_pt {
        push_style(&mut style, "top", &format!("{top:.2}pt"));
    }
    if let Some(size) = text.font_size_pt {
        push_style(&mut style, "font-size", &format!("{size:.2}pt"));
    }
    if let Some(color) = &text.color {
        push_style(&mut style, "color", color);
    }
    let family = text
        .font_family
        .as_deref()
        .map(pdf_font_family)
        .unwrap_or("sans-serif");
    push_style(&mut style, "font-family", family);
    if text.rotation_deg.is_some() {
        let outline = bounds_style(&text.bounds);
        out.push_str(&format!(
            "      <div class=\"rich-text-pdf-rotated-outline\" style=\"{}\"></div>\n",
            escape_attr(&outline)
        ));
    }
    out.push_str(&format!(
        "      <span class=\"rich-text-pdf-text\" style=\"{}\">{}</span>\n",
        escape_attr(&style),
        escape_html(&text.text)
    ));
}

fn render_runs(out: &mut String, runs: &[StyledChar]) {
    for (text, style) in group_runs(runs) {
        if text.is_empty() {
            continue;
        }
        let mut open = String::new();
        let mut close = String::new();
        if style.link.is_some() {
            open.push_str("<span class=\"rich-text-link\">");
            close.insert_str(0, "</span>");
        }
        if style.code {
            open.push_str("<code>");
            close.insert_str(0, "</code>");
        }
        if style.bold {
            open.push_str("<strong>");
            close.insert_str(0, "</strong>");
        }
        if style.italic {
            open.push_str("<em>");
            close.insert_str(0, "</em>");
        }
        if style.underline {
            open.push_str("<u>");
            close.insert_str(0, "</u>");
        }
        if style.strike {
            open.push_str("<s>");
            close.insert_str(0, "</s>");
        }
        out.push_str(&open);
        out.push_str(&escape_html(&text).replace('\n', "<br>"));
        out.push_str(&close);
    }
}

fn render_table(out: &mut String, table: &Table) {
    out.push_str("    <table>\n");
    for row in &table.rows {
        out.push_str("      <tr>");
        for cell in &row.cells {
            let tag = if cell.header { "th" } else { "td" };
            out.push_str(&format!("<{tag}>"));
            render_runs(out, &cell.content);
            out.push_str(&format!("</{tag}>"));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("    </table>\n");
}

fn render_image(out: &mut String, image: &Image) {
    let label = if image.alt.is_empty() {
        "[image]"
    } else {
        &image.alt
    };
    out.push_str("    <p><em>");
    out.push_str(&escape_html(label));
    out.push_str("</em></p>\n");
}

fn bounds_style(bounds: &PdfBox) -> String {
    let mut style = String::new();
    if let Some(value) = bounds.left_pt {
        push_style(&mut style, "left", &format!("{value:.2}pt"));
    }
    if let Some(value) = bounds.top_pt {
        push_style(&mut style, "top", &format!("{value:.2}pt"));
    }
    if let Some(value) = bounds.width_pt {
        push_style(&mut style, "width", &format!("{:.2}pt", value.max(1.0)));
    }
    if let Some(value) = bounds.height_pt {
        push_style(&mut style, "height", &format!("{:.2}pt", value.max(1.0)));
    }
    style
}

fn push_style(style: &mut String, name: &str, value: &str) {
    if !style.is_empty() {
        style.push(';');
    }
    style.push_str(name);
    style.push(':');
    style.push_str(value);
}

fn pdf_font_family(font_family: &str) -> &'static str {
    if font_family.to_ascii_lowercase().contains("courier")
        || font_family.to_ascii_lowercase().contains("monospace")
    {
        "ui-monospace, \"SFMono-Regular\", Consolas, monospace"
    } else {
        "sans-serif"
    }
}

fn inferred_page_width(page: &PdfPage) -> f32 {
    page.elements
        .iter()
        .filter_map(element_bounds)
        .filter_map(|bounds| Some(bounds.left_pt? + bounds.width_pt?))
        .fold(612.0, f32::max)
}

fn inferred_page_height(page: &PdfPage) -> f32 {
    page.elements
        .iter()
        .filter_map(element_bounds)
        .filter_map(|bounds| Some(bounds.top_pt? + bounds.height_pt?))
        .fold(792.0, f32::max)
}

fn element_bounds(element: &PdfElement) -> Option<&PdfBox> {
    match element {
        PdfElement::Text(text) => Some(&text.bounds),
        PdfElement::Image(image) => Some(&image.bounds),
        PdfElement::Shape(shape) => Some(&shape.bounds),
        PdfElement::Ink(ink) => Some(&ink.bounds),
        PdfElement::Link(link) => Some(&link.bounds),
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn editor_preview_css() -> &'static str {
    r#"
      :root {
        color: #222222;
        background: #f6efde;
        font-family: "Segoe UI", system-ui, sans-serif;
      }
      * {
        box-sizing: border-box;
      }
      html,
      body {
        margin: 0;
        min-width: max-content;
        background: #f6efde;
      }
      .rich-text-preview {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 24px;
        min-height: 100vh;
        padding: 24px;
      }
      .rich-text-pdf-page {
        position: relative;
        overflow: hidden;
        flex: 0 0 auto;
        background: white;
        border: 1px solid #b4b4b4;
        border-radius: 2px;
      }
      .rich-text-pdf-shape,
      .rich-text-pdf-image,
      .rich-text-pdf-text,
      .rich-text-pdf-ink,
      .rich-text-pdf-link,
      .rich-text-pdf-rotated-outline {
        position: absolute;
        box-sizing: border-box;
      }
      .rich-text-pdf-text {
        white-space: pre;
        line-height: 1;
        font-weight: 400;
        font-style: normal;
      }
      .rich-text-pdf-image {
        display: grid;
        place-items: center;
        overflow: hidden;
        border: 1px solid #a5a5a5;
        background: #ebebeb;
        color: #5a5a5a;
        font: 10px/1.2 ui-monospace, "SFMono-Regular", Consolas, monospace;
        text-align: center;
        padding: 4px;
      }
      .rich-text-pdf-ink {
        border: 1px solid #787878;
      }
      .rich-text-pdf-link {
        outline: 1px solid #3b6df0;
      }
      .rich-text-pdf-rotated-outline {
        border: 0.75px solid #4673b4;
      }
      p,
      h1,
      h2,
      h3,
      h4,
      h5,
      h6,
      blockquote,
      pre,
      table,
      hr {
        width: min(760px, 100%);
      }
      p {
        font-size: 17px;
        line-height: 1.55;
      }
      blockquote {
        padding-left: 12px;
        border-left: 1px solid #9a9a9a;
      }
      table {
        border-collapse: collapse;
      }
      td,
      th {
        padding: 4px 12px;
        border: 1px solid #d3d3d3;
      }
      .rich-text-link {
        color: #3b6df0;
        text-decoration: underline;
      }
      .rich-text-note {
        color: #777777;
        font-size: 12px;
      }
      .rich-text-raw {
        white-space: pre-wrap;
      }
    "#
}
