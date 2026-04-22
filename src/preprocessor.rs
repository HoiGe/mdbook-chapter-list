use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use std::fmt::Write;
use std::path::{Path, PathBuf};

pub struct ChapterList;

impl ChapterList {
    pub fn new() -> Self {
        Self
    }
}

fn add_nested(listing: &mut String, indent: usize, chapter: &Chapter, src_dir: &Path) {
    for sub in &chapter.sub_items {
        if let BookItem::Chapter(sub_chapter) = sub {
            // 确定基准目录：父章节有路径则取其父目录，否则用 src_dir
            let base_dir = chapter
                .path
                .as_ref()
                .and_then(|p| p.parent())
                .unwrap_or(src_dir);

            // 输出当前子章节的行
            if let Some(sub_path) = &sub_chapter.path {
                let relpath = pathdiff::diff_paths(sub_path, base_dir)
                    .unwrap_or_else(|| sub_path.to_path_buf());
                writeln!(
                    listing,
                    "{} - [{}]({})",
                    "   ".repeat(indent),
                    sub_chapter.name,
                    relpath.display()
                )
                .unwrap();
            } else {
                // 没有路径的章节只输出纯文本
                writeln!(listing, "{}- {}", "   ".repeat(indent), sub_chapter.name).unwrap();
            }

            // 递归处理子章节，基准目录保持不变
            add_nested(listing, indent + 1, sub_chapter, src_dir);
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

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // Look in each book chapter if we have the replacement mark.
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if !chapter.content.contains("<!-- chapter-list -->") {
                    chapter.content.push_str("\n\n<!-- chapter-list -->");
                }
                // Generate the sub-chapter list.
                let mut listing = String::new();
                let src_dir = ctx.root.join(&ctx.config.book.src);
                add_nested(&mut listing, 0, chapter, &src_dir);

                // Insert the sub-chapter list if asked for.
                let content = chapter.content.replace("<!-- chapter-list -->", &listing);
                chapter.content = content;
            }
        });
        Ok(book)
    }
}
