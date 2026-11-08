---
slug: welcome
title: "Welcome: A Tour of the Content Pipeline"
description: "A sample post that exercises every markdown feature the build supports."
theme: dusty_trail
date: 2026-11-08
---

This post is sample content. It shows what `build.rs` does to a markdown file in
`content/`, so you can delete it once you've written your own.

Everything below is processed at compile time: the server never reads a file to render a page.

---

## Callouts

::: tip
`::: tip`, `::: note` and `::: warning` blocks become styled callouts.
:::

::: note
Horizontal rules (`---`) split a post into separate cards.
:::

::: warning
Angle brackets outside code are escaped by default (`escape_angles: true`), so <this> shows up as text.
:::

---

## Code

Fenced code blocks are highlighted at build time with syntect:

```rust
fn main() {
    let stripes = ["red", "orange", "gold"];
    for stripe in stripes {
        println!("{stripe}");
    }
}
```

---

## Links

Headings get anchor links, so you can point at [the Links section](#links) directly.

Special link prefixes expand at build time:

- [This post as raw markdown](raw:/posts/welcome) (`raw:/posts/welcome`)
- [View a file in the browser](file:/example.sh) (`file:/example.sh`)
- [Download the same file](download:example.sh) (`download:example.sh`)

Files under `public/files/` are served at `/files/`.
