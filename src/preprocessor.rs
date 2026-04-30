use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

pub struct ChapterList;

const CHAPTER_LIST_MARKER: &str = "<!-- chapter-list -->";

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct ChapterListConfig {
    ignored_files: Vec<PathBuf>,
}

#[derive(Debug, Default)]
struct IgnoredFiles {
    paths: HashSet<String>,
}

impl ChapterList {
    pub fn new() -> Self {
        Self
    }
}

impl IgnoredFiles {
    fn new(paths: Vec<PathBuf>, src_dir: &Path) -> Self {
        let mut ignored = Self::default();
        let src_dir = normalize_path(src_dir);
        let src_dir_prefix = format!("{src_dir}/");

        for path in paths {
            let path = normalize_path(&path);
            ignored.paths.insert(path.clone());

            if let Some(stripped) = path.strip_prefix(&src_dir_prefix) {
                ignored.paths.insert(stripped.to_string());
            }
        }

        ignored
    }

    fn contains_chapter(&self, chapter: &Chapter) -> bool {
        chapter
            .source_path
            .iter()
            .chain(chapter.path.iter())
            .any(|path| self.paths.contains(&normalize_path(path)))
    }
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::CurDir => None,
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".to_string()),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn chapter_link_path(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();

    if path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.eq_ignore_ascii_case("readme"))
        .unwrap_or(false)
    {
        path.set_file_name("index.md");
    }

    path
}

fn add_nested(
    listing: &mut String,
    indent: usize,
    chapter: &Chapter,
    link_base_dir: &Path,
    ignored_files: &IgnoredFiles,
) {
    for sub in &chapter.sub_items {
        if let BookItem::Chapter(sub_chapter) = sub {
            if ignored_files.contains_chapter(sub_chapter) {
                add_nested(listing, indent, sub_chapter, link_base_dir, ignored_files);
                continue;
            }

            // Add the current sub-chapter to the generated list.
            if let Some(sub_path) = &sub_chapter.path {
                let link_path = chapter_link_path(sub_path);
                let relpath = pathdiff::diff_paths(&link_path, link_base_dir).unwrap_or(link_path);
                listing.push_str(&format!(
                    "{} - [{}]({})\n",
                    "   ".repeat(indent),
                    sub_chapter.name,
                    relpath.display()
                ));
            } else {
                // Draft chapters do not have paths, so render plain text.
                listing.push_str(&format!("{}- {}\n", "   ".repeat(indent), sub_chapter.name));
            }

            // Recurse into nested sub-chapters.
            add_nested(
                listing,
                indent + 1,
                sub_chapter,
                link_base_dir,
                ignored_files,
            );
        }
    }
}

fn update_chapter(chapter: &mut Chapter, ignored_files: &IgnoredFiles) {
    if ignored_files.contains_chapter(chapter) {
        return;
    }

    // Generate the sub-chapter list.
    let link_base_dir = chapter
        .path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| Path::new(""));
    let mut listing = String::new();
    add_nested(&mut listing, 0, chapter, link_base_dir, ignored_files);

    if listing.is_empty() {
        return;
    }

    // Insert the sub-chapter list at the first marker, or append it.
    if chapter.content.contains(CHAPTER_LIST_MARKER) {
        chapter.content = chapter.content.replacen(CHAPTER_LIST_MARKER, &listing, 1);
    } else {
        chapter.content.push_str("\n\n");
        chapter.content.push_str(&listing);
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
        let config: ChapterListConfig = ctx
            .config
            .get_deserialized_opt("preprocessor.chapter-list")?
            .unwrap_or_default();
        let ignored_files = IgnoredFiles::new(config.ignored_files, &ctx.config.book.src);

        // Update each chapter that has visible sub-chapters.
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                update_chapter(chapter, &ignored_files);
            }
        });
        Ok(book)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignored_files_match_source_relative_and_normalized_paths() {
        let ignored = IgnoredFiles::new(
            vec![
                PathBuf::from("src/internal/notes.md"),
                PathBuf::from("index.md"),
            ],
            Path::new("src"),
        );
        let notes = Chapter::new(
            "Ignored File",
            String::new(),
            "internal/notes.md",
            Vec::new(),
        );
        let mut chapter = Chapter::new("Index File", String::new(), "README.md", Vec::new());
        chapter.path = Some(PathBuf::from("index.md"));

        assert!(ignored.contains_chapter(&notes));
        assert!(ignored.contains_chapter(&chapter));
    }

    #[test]
    fn updates_content_only_when_there_is_a_visible_listing() {
        let ignored = IgnoredFiles::default();
        let mut leaf = Chapter::new("Leaf", "# Leaf".to_string(), "leaf.md", Vec::new());
        let mut marked = Chapter::new(
            "Marked",
            "<!-- chapter-list -->\n\n<!-- chapter-list -->".to_string(),
            "marked.md",
            Vec::new(),
        );
        let mut appended = Chapter::new(
            "Container",
            "# Container".to_string(),
            "parent.md",
            Vec::new(),
        );

        marked.sub_items.push(chapter_item("Item", "item.md"));
        appended.sub_items.push(chapter_item("Item", "item.md"));

        update_chapter(&mut leaf, &ignored);
        update_chapter(&mut marked, &ignored);
        update_chapter(&mut appended, &ignored);

        assert_eq!(leaf.content, "# Leaf");
        assert_eq!(
            marked.content,
            " - [Item](item.md)\n\n\n<!-- chapter-list -->"
        );
        assert_eq!(appended.content, "# Container\n\n - [Item](item.md)\n");
    }

    #[test]
    fn renders_nested_draft_and_relative_links() {
        let ignored = IgnoredFiles::default();
        let mut parent = Chapter::new("Root", String::new(), "guide/root.md", Vec::new());
        let mut child = chapter("Section", "guide/section.md");
        child
            .sub_items
            .push(chapter_item("Nested", "guide/nested.md"));

        parent.sub_items.push(BookItem::Chapter(child));
        parent
            .sub_items
            .push(chapter_item("External", "reference/external.md"));
        parent
            .sub_items
            .push(chapter_item("Index", "guide/readme.md"));
        parent
            .sub_items
            .push(BookItem::Chapter(Chapter::new_draft("Draft", Vec::new())));

        let mut listing = String::new();
        add_nested(&mut listing, 0, &parent, Path::new("guide"), &ignored);

        assert_eq!(
            listing,
            " - [Section](section.md)\n    - [Nested](nested.md)\n - [External](../reference/external.md)\n - [Index](index.md)\n- Draft\n"
        );
    }

    #[test]
    fn ignored_chapters_are_skipped_but_their_children_remain_visible() {
        let ignored = IgnoredFiles::new(
            vec![
                PathBuf::from("parent.md"),
                PathBuf::from("unicode/章节/index.md"),
            ],
            Path::new("src"),
        );
        let mut parent = Chapter::new(
            "Ignored Root",
            "# Ignored Root".to_string(),
            "parent.md",
            Vec::new(),
        );
        let mut container = Chapter::new(
            "Ignored Container",
            String::new(),
            "unicode/章节/index.md",
            Vec::new(),
        );
        container
            .sub_items
            .push(chapter_item("Visible Item", "unicode/章节/item.md"));

        parent
            .sub_items
            .push(chapter_item("Hidden Item", "hidden.md"));
        update_chapter(&mut parent, &ignored);

        let mut visible_parent =
            Chapter::new("Visible Root", String::new(), "index.md", Vec::new());
        visible_parent.sub_items.push(BookItem::Chapter(container));

        let mut listing = String::new();
        add_nested(&mut listing, 0, &visible_parent, Path::new(""), &ignored);

        assert_eq!(parent.content, "# Ignored Root");
        assert_eq!(listing, " - [Visible Item](unicode/章节/item.md)\n");
    }

    fn chapter(name: &str, path: &str) -> Chapter {
        Chapter::new(name, String::new(), path, Vec::new())
    }

    fn chapter_item(name: &str, path: &str) -> BookItem {
        BookItem::Chapter(chapter(name, path))
    }
}
