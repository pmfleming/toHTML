use crate::{Table, TableCell};

use crate::html::attrs::{push_end_tag, push_number_attr};
use crate::html::inlines::render_inlines;

pub(super) fn render_table(html: &mut String, table: &Table) {
    html.push_str("    <table>\n");
    render_caption(html, table);
    for row in &table.rows {
        html.push_str("      <tr>");
        for cell in &row.cells {
            render_table_cell(html, cell);
        }
        html.push_str("</tr>\n");
    }
    html.push_str("    </table>\n");
}

fn render_caption(html: &mut String, table: &Table) {
    if let Some(caption) = &table.caption {
        html.push_str("      <caption>");
        render_inlines(html, caption);
        html.push_str("</caption>\n");
    }
}

fn render_table_cell(html: &mut String, cell: &TableCell) {
    let tag = if cell.header { "th" } else { "td" };
    html.push('<');
    html.push_str(tag);
    push_number_attr(html, "colspan", span_attr(cell.colspan));
    push_number_attr(html, "rowspan", span_attr(cell.rowspan));
    html.push('>');
    render_inlines(html, &cell.content);
    push_end_tag(html, tag);
}

fn span_attr(span: u16) -> Option<u64> {
    (span > 1).then_some(u64::from(span))
}
