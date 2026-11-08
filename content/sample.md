---
slug: sample
title: "Style Sample"
description: "Design system showcase for Dusty Trail theme"
theme: dusty_trail
date: 2025-01-04
draft: true
escape_angles: false
---

This page demonstrates all available styling elements in the Dusty Trail theme. Use it to test and refine the design system.

---

## Typography

### Headings

# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

### Body Text

This is a paragraph with **bold text**, *italic text*, and `inline code`. You can also combine them for ***bold italic*** text. Links look [like this](/sample).

A second paragraph to show spacing. The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.

### Lists

**Unordered list:**
- First item
- Second item with more text to show wrapping behavior
- Third item
  - Nested item one
  - Nested item two

**Ordered list:**
1. First step
2. Second step
3. Third step

---

## Code

Inline code: `const greeting = "Hello, Dusty Trail!"`

Code block:

```rust
fn main() {
    let colors = ["red", "orange", "gold"];
    for color in &colors {
        println!("Sunset: {}", color);
    }
}
```

---

## Callouts

::: tip
This is a tip callout. Use it for helpful suggestions or best practices.
:::

::: note
This is a note callout. Use it for additional context or information.
:::

::: warning
This is a warning callout. Use it for important caveats or potential issues.
:::

---

## Buttons

<p class="demo-label">Color variants</p>
<div class="button-row">
  <button class="btn btn-red">Red</button>
  <button class="btn btn-orange">Orange</button>
  <button class="btn btn-gold">Gold</button>
  <button class="btn">Neutral</button>
  <button class="btn btn-ghost">Ghost</button>
</div>

<p class="demo-label" style="margin-top: 1.5rem;">As links</p>
<div class="button-row">
  <a href="#" class="btn btn-red">Link Button</a>
  <a href="#" class="btn btn-orange">Link Button</a>
</div>

---

## Stripes

### Horizontal Stripes

<p class="demo-label">stripe-sm (6px)</p>
<div class="stripe-tri stripe-sm">
  <span class="s-red"></span>
  <span class="s-orange"></span>
  <span class="s-gold"></span>
</div>

<p class="demo-label" style="margin-top: 1rem;">stripe-md (12px)</p>
<div class="stripe-tri stripe-md">
  <span class="s-red"></span>
  <span class="s-orange"></span>
  <span class="s-gold"></span>
</div>

<p class="demo-label" style="margin-top: 1rem;">stripe-lg (18px)</p>
<div class="stripe-tri stripe-lg">
  <span class="s-red"></span>
  <span class="s-orange"></span>
  <span class="s-gold"></span>
</div>

### Vertical Stripes

<div style="display: flex; gap: 1.5rem; margin-top: 1rem;">
  <div>
    <p class="demo-label">stripe-v-sm</p>
    <div class="stripe-tri-vertical stripe-v-sm" style="height: 80px;">
      <span class="s-red"></span>
      <span class="s-orange"></span>
      <span class="s-gold"></span>
    </div>
  </div>
  <div>
    <p class="demo-label">stripe-v-md</p>
    <div class="stripe-tri-vertical stripe-v-md" style="height: 80px;">
      <span class="s-red"></span>
      <span class="s-orange"></span>
      <span class="s-gold"></span>
    </div>
  </div>
  <div>
    <p class="demo-label">stripe-v-lg</p>
    <div class="stripe-tri-vertical stripe-v-lg" style="height: 80px;">
      <span class="s-red"></span>
      <span class="s-orange"></span>
      <span class="s-gold"></span>
    </div>
  </div>
</div>

---

## Color Palette

### Accent Colors

<div class="palette-row">
  <div class="swatch" style="background: var(--accent-red); color: #fff;">
    <span class="swatch-label">Red</span>
    <span>hsl(5,62%,60%)</span>
  </div>
  <div class="swatch" style="background: var(--accent-orange); color: #000;">
    <span class="swatch-label">Orange</span>
    <span>hsl(25,62%,60%)</span>
  </div>
  <div class="swatch" style="background: var(--accent-gold); color: #000;">
    <span class="swatch-label">Gold</span>
    <span>hsl(45,62%,60%)</span>
  </div>
</div>

### Warm Neutrals

<div class="palette-row">
  <div class="swatch" style="background: var(--neutral-50); color: #333;">
    <span class="swatch-label">50</span>
  </div>
  <div class="swatch" style="background: var(--neutral-100); color: #333;">
    <span class="swatch-label">100</span>
  </div>
  <div class="swatch" style="background: var(--neutral-200); color: #333;">
    <span class="swatch-label">200</span>
  </div>
  <div class="swatch" style="background: var(--neutral-300); color: #333;">
    <span class="swatch-label">300</span>
  </div>
  <div class="swatch" style="background: var(--neutral-400); color: #fff;">
    <span class="swatch-label">400</span>
  </div>
  <div class="swatch" style="background: var(--neutral-500); color: #fff;">
    <span class="swatch-label">500</span>
  </div>
  <div class="swatch" style="background: var(--neutral-600); color: #fff;">
    <span class="swatch-label">600</span>
  </div>
  <div class="swatch" style="background: var(--neutral-700); color: #fff;">
    <span class="swatch-label">700</span>
  </div>
  <div class="swatch" style="background: var(--neutral-800); color: #fff;">
    <span class="swatch-label">800</span>
  </div>
  <div class="swatch" style="background: var(--neutral-900); color: #fff;">
    <span class="swatch-label">900</span>
  </div>
</div>

---

## Cards

### Card with Stripe

<div class="card" style="max-width: 360px;">
  <div class="stripe-tri stripe-sm">
    <span class="s-red"></span>
    <span class="s-orange"></span>
    <span class="s-gold"></span>
  </div>
  <div class="card-body">
    <h4 class="card-title">Card Title</h4>
    <p class="card-text">The neutral background lets content breathe while the stripe adds personality at a glance.</p>
    <button class="btn btn-orange">View Details</button>
  </div>
</div>

### Side Stripe Layout

<div class="side-stripe-container" style="max-width: 400px; margin-top: 1.5rem;">
  <div class="stripe-tri-vertical stripe-v-md" style="min-height: 100%;">
    <span class="s-red"></span>
    <span class="s-orange"></span>
    <span class="s-gold"></span>
  </div>
  <div class="side-stripe-content">
    <h4 style="margin-top: 0;">A Note on Design</h4>
    <p style="font-size: 0.9rem; margin-bottom: 0;">The vertical bands read as intentional and styled without demanding attention. Works well for tips, quotes, or secondary information.</p>
  </div>
</div>

---

## Inset Panels

<p class="demo-label">Light inset</p>
<div class="inset">
This is an inset panel. Good for secondary information, code snippets, or callouts that need to recede visually.
</div>

<p class="demo-label" style="margin-top: 1.5rem;">Dark inset</p>
<div class="inset inset-dark">
// Dark variant for code or terminal-style content<br>
The warm undertones keep it cozy rather than cold.
</div>

---

## Combined Examples

Here's how elements work together in typical content flow:

::: tip
When combining elements, maintain visual hierarchy. Callouts should enhance, not overwhelm.
:::

A paragraph following the callout with a [link to somewhere](#) and some `inline code` that demonstrates how text flows normally after a callout block.

<div class="button-row" style="margin: 1.5rem 0;">
  <a href="/" class="btn btn-gold">Back Home</a>
  <a href="/about" class="btn btn-ghost">Learn More</a>
</div>

And a final stripe to close things out:

<div class="stripe-tri stripe-lg" style="margin-top: 1.5rem;">
  <span class="s-red"></span>
  <span class="s-orange"></span>
  <span class="s-gold"></span>
</div>
