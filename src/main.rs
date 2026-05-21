use tohtml::{render_html, Block, Document};

fn main() {
    let mut document = Document::with_title("toHTML");
    document.blocks.push(Block::heading(2, "Starter CLI"));
    document.blocks.push(Block::paragraph(
        "The converter pipeline is scaffolded. Format readers come next.",
    ));

    print!("{}", render_html(&document));
}
