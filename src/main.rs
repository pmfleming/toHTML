use tohtml::{render_html, Block, Document};

fn main() {
    let document = Document {
        title: Some("toHTML".to_string()),
        blocks: vec![
            Block::Heading {
                level: 2,
                text: "Starter CLI".to_string(),
            },
            Block::Paragraph(
                "The converter pipeline is scaffolded. Format readers come next.".to_string(),
            ),
        ],
    };

    print!("{}", render_html(&document));
}
