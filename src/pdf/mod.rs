mod cmap;
mod fonts;
mod form;
mod graphics;
mod hex;
mod images;
mod layout;
#[cfg(test)]
mod layout_tests;
mod links;
mod metadata;
mod object;
mod postprocess;
mod repair;
mod streams;
mod struct_tree;
#[cfg(test)]
mod tests;
mod text;
mod visual;

use crate::ConvertError;
use crate::{Block, ConversionWarning, Document, PageBreak};
use crate::{PagePlaceholder, PlaceholderReason, SourceFormat};

use object::PdfObjects;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfConversionOptions {
    pub include_images: bool,
}

impl Default for PdfConversionOptions {
    fn default() -> Self {
        Self {
            include_images: true,
        }
    }
}

pub fn pdf_to_document(bytes: &[u8]) -> Result<Document, ConvertError> {
    pdf_to_document_with_options(bytes, PdfConversionOptions::default())
}

pub fn pdf_to_document_with_options(
    bytes: &[u8],
    options: PdfConversionOptions,
) -> Result<Document, ConvertError> {
    let extraction = streams::document_pages(bytes)?;
    let font_cmaps = cmap::font_cmaps(bytes)?;
    let font_metrics = fonts::font_metrics(bytes);
    let objects = PdfObjects::parse(bytes);
    let struct_roles = struct_tree::role_map(&objects);
    let mut document = Document::new();
    document.metadata.source_format = Some(SourceFormat::Pdf);
    document.metadata.title = metadata::document_title(&objects);
    document.metadata.language = metadata::document_language(&objects);
    document.warnings.extend(
        extraction
            .warnings
            .into_iter()
            .map(|message| ConversionWarning {
                message,
                source: None,
            }),
    );

    let total_pages = extraction.pages.len();
    let mut visual_pages = Vec::new();
    for (page_index, page) in extraction.pages.iter().enumerate() {
        let mut page_blocks = Vec::new();
        let mut page_segments = Vec::new();
        let mut semantic_segments = Vec::new();
        let mut page_shapes = Vec::new();
        let mut page_font_cmaps = font_cmaps.clone();
        page_font_cmaps.extend(cmap::font_cmaps_for_resources(
            bytes,
            &objects,
            &page.font_resources,
        )?);
        let mut page_font_metrics = font_metrics.clone();
        page_font_metrics.extend(fonts::font_metrics_for_resources(
            bytes,
            &page.font_resources,
        ));
        for stream in &page.streams {
            page_shapes.extend(graphics::extract_rectangles(stream));
        }
        let (form_shapes, form_paths) =
            form::form_xobject_graphics(&objects, &page.image_resources, &page.streams);
        page_shapes.extend(form_shapes);
        repair::remove_redundant_header_hairlines(page.height.unwrap_or(842.0), &mut page_shapes);
        let page_images = if options.include_images {
            images::extract_page_images(
                bytes,
                &objects,
                &page.streams,
                &page.image_resources,
                page.page_number,
                &mut document.warnings,
            )
        } else {
            Vec::new()
        };
        let mut page_paths = page
            .streams
            .iter()
            .flat_map(|stream| {
                graphics::extract_paths_with_shading_fills(stream, &page.shading_resources)
            })
            .chain(form_paths)
            .map(|path| visual::VisualPath {
                commands: path.commands,
                fill: path.fill,
                stroke: path.stroke,
                stroke_width: path.stroke_width,
                stroke_dasharray: path.stroke_dasharray,
            })
            .collect::<Vec<_>>();
        page_paths.extend(page.ink_annotations.iter().flat_map(|annotation| {
            let stroke = annotation
                .color
                .clone()
                .unwrap_or_else(|| "#000000".to_string());
            annotation
                .paths
                .iter()
                .map(move |points| visual::VisualPath {
                    commands: ink_path_commands(points),
                    fill: None,
                    stroke: Some(stroke.clone()),
                    stroke_width: annotation.width,
                    stroke_dasharray: None,
                })
        }));

        let combined_stream = combined_page_stream(&page.streams);
        if !combined_stream.is_empty() {
            page_segments = text::extract_segments_with_fonts(
                &combined_stream,
                &page_font_cmaps,
                &page_font_metrics,
                &struct_roles,
            );
            page_segments.extend(form::form_xobject_text_segments(
                &objects,
                &page.image_resources,
                &page.streams,
                &page_font_cmaps,
                &page_font_metrics,
                &struct_roles,
            ));
            text::repair_segment_text(&mut page_segments);
            repair::restore_centered_page_number_markers(
                page.width.unwrap_or(612.0),
                page.height.unwrap_or(842.0),
                &mut page_segments,
            );
            repair::split_segments_at_column_gaps(page.width.unwrap_or(612.0), &mut page_segments);
            repair::split_multicolumn_sublabels(page.height.unwrap_or(842.0), &mut page_segments);
            repair::tighten_overlapping_text_widths(&mut page_segments);
            semantic_segments = text::non_artifact_segments(&page_segments);
            page_blocks.extend(layout::blocks_from_segments(&semantic_segments));
        }
        layout::add_content_images_to_blocks(
            &mut page_blocks,
            &semantic_segments,
            &page_images,
            page.width.unwrap_or(612.0),
            page.height.unwrap_or(842.0),
            page.page_number,
        );
        if !page_segments.is_empty()
            || !page_shapes.is_empty()
            || !page_images.is_empty()
            || !page_paths.is_empty()
        {
            visual_pages.push(visual::VisualPage {
                page_number: page.page_number,
                width: page.width,
                height: page.height,
                segments: page_segments,
                shapes: page_shapes,
                images: page_images,
                paths: page_paths,
                links: page
                    .link_annotations
                    .iter()
                    .map(|annotation| visual::VisualLink {
                        href: annotation.uri.clone(),
                        x: annotation.rect.0,
                        y: annotation.rect.1,
                        width: annotation.rect.2,
                        height: annotation.rect.3,
                    })
                    .collect(),
            });
        }
        if page_blocks.is_empty() {
            page_blocks.push(Block::PagePlaceholder(PagePlaceholder {
                page_number: Some(page.page_number),
                reason: PlaceholderReason::NonExtractable,
                source: None,
            }));
        }
        document.blocks.extend(page_blocks);
        if total_pages > 1 && page_index + 1 < total_pages {
            document.blocks.push(Block::PageBreak(PageBreak {
                page_number: Some(page.page_number),
                source: None,
            }));
        }
    }
    document.blocks = postprocess::blocks(document.blocks, extraction.page_count);

    if document.blocks.is_empty() {
        add_empty_pdf_placeholder(&mut document, extraction.page_count);
    }
    if !document
        .blocks
        .iter()
        .any(|block| !matches!(block, Block::PagePlaceholder(_)))
        && !document
            .warnings
            .iter()
            .any(|warning| warning.message.contains("PDF contained no selectable text"))
    {
        document.warnings.push(ConversionWarning {
            message: "PDF contained no selectable text in supported content streams".to_string(),
            source: None,
        });
    }
    add_image_text_warning(&mut document, bytes);
    links::apply_detected_links(&mut document.blocks, &mut document.warnings, bytes);
    document.blocks = postprocess::link_artifacts(document.blocks);
    document.metadata.visual_html = visual::render_pages(&visual_pages);

    Ok(document)
}

fn combined_page_stream(streams: &[Vec<u8>]) -> Vec<u8> {
    let mut combined = Vec::new();
    for stream in streams {
        if !combined.is_empty() {
            combined.push(b'\n');
        }
        combined.extend_from_slice(stream);
    }
    combined
}

fn ink_path_commands(points: &[(f32, f32)]) -> Vec<graphics::PathCommand> {
    let mut commands = Vec::new();
    for (index, (x, y)) in points.iter().enumerate() {
        if index == 0 {
            commands.push(graphics::PathCommand::MoveTo(*x, *y));
        } else {
            commands.push(graphics::PathCommand::LineTo(*x, *y));
        }
    }
    commands
}

fn add_empty_pdf_placeholder(document: &mut Document, pages: usize) {
    for page in 1..=pages.max(1) {
        document
            .blocks
            .push(Block::PagePlaceholder(PagePlaceholder {
                page_number: Some(page as u32),
                reason: PlaceholderReason::NonExtractable,
                source: None,
            }));
    }
    document.warnings.push(ConversionWarning {
        message: "PDF contained no selectable text in supported content streams".to_string(),
        source: None,
    });
}

fn add_image_text_warning(document: &mut Document, bytes: &[u8]) {
    if !has_image_xobject(bytes) {
        return;
    }

    document.warnings.push(ConversionWarning {
        message:
            "PDF contains image content that may include non-selectable text; OCR is not performed"
                .to_string(),
        source: None,
    });
}

fn has_image_xobject(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    text.contains("/Subtype /Image") || text.contains("/Subtype/Image")
}
