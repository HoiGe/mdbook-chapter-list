# Chapter Listing plugin for mdbook

The `chapter-list` preprocessor supports adding sub-chapter lists to documents.

## Example

With a `SUMMARY.md` file like:

```
- [Zoo Animals](./zoo.md)
  - [Large Cats](./cats.md)
    - [Lion](./lion.md)
    - [Tiger](./tiger.md)
  - [Zebra](./zebra.md)
  - [Turtle](./turtle.md)
```

Each chapter gets a sub-chapter list by default. For example, if `zoo.md`
contains:

```md
## Animals in the Zoo
```

The zoo.md file would be updated to:

```md
## Animals in the Zoo

 - [Large Cats](cats.md)
    - [Lion](lion.md)
    - [Tiger](tiger.md)
 - [Zebra](zebra.md)
 - [Turtle](turtle.md)
```

To choose where the generated list is inserted, add `<!-- chapter-list -->` to
the chapter. The first marker is replaced with the generated list. If the marker
is not present, the list is appended to the end of the chapter. Chapters without
sub-chapters are left unchanged.


## Installation

Firstly add the following to your book's manifest file (usually `book.toml`)

```toml
[preprocessor.chapter-list]
```

To skip files, add `ignored-files`. Paths are relative to the book source
directory:

```toml
[preprocessor.chapter-list]
ignored-files = ["draft.md", "internal/notes.md"]
```
