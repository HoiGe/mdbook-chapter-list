use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::fmt::Write;

pub struct ChapterList;

impl ChapterList {
    pub fn new() -> Self {
        Self
    }
}

/// Add a nested ordered list of sub-chapters for the given chapter.
fn add_nested(listing: &mut String, indent: usize, chapter: &Chapter) {
    for (i, sub) in chapter.sub_items.iter().enumerate() {
        if let BookItem::Chapter(sub_chapter) = sub {
            let relpath = pathdiff::diff_paths(
                sub_chapter.path.clone().unwrap().as_path(),
                chapter.path.clone().unwrap().parent().unwrap(),
            )
            .unwrap();

            // Create the markdown ordered item string.
            writeln!(
                listing,
                "{} - [{}]({})",
                "   ".repeat(indent),
                sub_chapter.name,
                relpath.display()
            )
            .unwrap();

            // Recurse down the hierarchy.
            add_nested(listing, indent + 1, sub_chapter);
        }
    }
}

impl Preprocessor for ChapterList {
    fn name(&self) -> &str {
        "chapter-list"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // Look in each book chapter if we have the replacement mark.
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if !chapter.content.contains("<!-- chapter-list -->") {
                    chapter.content.push_str("\n\n<!-- chapter-list -->");
                }
                // Generate the sub-chapter list.
                let mut listing = String::new();
                add_nested(&mut listing, 0, chapter);

                // Insert the sub-chapter list if asked for.
                let content = chapter.content.replace("<!-- chapter-list -->", &listing);
                chapter.content = content;
            }
        });
        Ok(book)
    }
}
