---
title: Strings, hex and named colors
sidebar_position: 1
description: Parse and format hex, handle full CSS Color Level 4 syntax, look up the 148 CSS/SVG names, and turn a color back into its keyword.
---

# Strings, hex and named colors

There are two parsers with different reach. **Hex** (`plt_from_hex`, `plt_to_hex`) is palette's
own reader and writer: compact, strict, no color names. **CSS** (`plt_css_parse`,
`plt_css_serialize`) goes through csscolorparser and understands the full CSS Color Level 4
grammar, including `lab()`, `oklch()` and `color(display-p3 …)`.

Both parsers produce **sRGB** colors, because both formats encode sRGB. Nothing ever returns an
`oklch` value from a string: parse first, then `plt_convert` if you want another space.

## Hex

### `plt_from_hex(text)`

**Signature**: `plt_from_hex(text VARCHAR) -> plt_color`

Parses `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`; the leading `#` is optional. Short forms expand
each digit by duplication (`#f80` is `#ff8800`), and a 4- or 8-digit value carries alpha. Text that
is not a hex color returns **NULL**, not an error — so it composes with `WHERE` and
`COALESCE`:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_from_hex('#f80')) AS short,
       plt_to_hex(plt_from_hex('#ff800080')) AS with_alpha,
       plt_to_hex(plt_from_hex('ff8000')) AS without_hash,
       plt_from_hex('orange') IS NULL AS unknown_is_null;
```

### `plt_to_hex(color)`

**Signature**: `plt_to_hex(color plt_color) -> VARCHAR`

Converts to sRGB, quantizes to 8 bits per channel and writes `#rrggbb`; when alpha is not exactly 1
it writes `#rrggbbaa` instead. Out-of-gamut components are clamped, so this is always a valid hex
string:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_srgb(1, 0.5, 0)) AS orange,
       plt_to_hex(plt_color('srgb', 1, 0.5, 0, 0.5)) AS translucent,
       plt_to_hex(plt_oklch(0.7, 0.1, 120)) AS from_oklch;
```

## CSS syntax

### `plt_css_parse(text)`

**Signature**: `plt_css_parse(text VARCHAR) -> plt_color`

Parses any CSS Color Level 4 color into an sRGB color with alpha. That covers named colors, all
hex forms, `rgb()`, `rgba()`, `hsl()`, `hsla()`, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()` and
`color(...)` with the usual color spaces. Separators may be commas or spaces, and the alpha may be
a number or a percentage.

Unparsable text returns **NULL**:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_css_parse('hsl(120 100% 50% / 0.5)')) AS hsl,
       plt_to_hex(plt_css_parse('rgb(255, 128, 0)')) AS rgb,
       plt_to_hex(plt_css_parse('oklch(0.7 0.15 250)')) AS oklch,
       plt_css_parse('not a color') IS NULL AS invalid_is_null;
```

### `plt_css_is_valid(text)`

**Signature**: `plt_css_is_valid(text VARCHAR) -> BOOLEAN`

Whether `plt_css_parse` would succeed. Use it to validate a column before parsing, or to keep the
invalid rows visible in a report:

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_is_valid('#ff0000') AS hex_ok,
       plt_css_is_valid('rgb(300 0 0)') AS clampable,
       plt_css_is_valid('rgb(nope)') AS broken;
```

### `plt_css_serialize(color, format)`

**Signature**: `plt_css_serialize(color plt_color, format VARCHAR) -> VARCHAR`

Writes a color as CSS text, converting to sRGB first. The eight formats are the eight functions
csscolorparser offers: `hex`, `rgb`, `hsl`, `hwb`, `lab`, `lch`, `oklab`, `oklch`.

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_formats() AS formats;
```

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_serialize(plt_srgb(1, 0.5, 0), 'hex') AS hex,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'rgb') AS rgb,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'oklch') AS oklch,
       plt_css_serialize(plt_srgb(1, 0.5, 0), 'lab') AS lab;
```

A full round trip through CSS text stays in sRGB and keeps alpha:

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_parse(plt_css_serialize(c, 'oklch')) AS reparsed,
       plt_to_hex(plt_css_parse(plt_css_serialize(c, 'oklch'))) AS hex
FROM (SELECT plt_color('srgb', 0.2, 0.4, 0.6, 0.5) AS c);
```

An unknown format fails the statement:

```sql {"type":"duckfn","expect":"error"}
SELECT plt_css_serialize(plt_srgb(1, 0, 0), 'hsv');
```

### `plt_css_name(color)`

**Signature**: `plt_css_name(color plt_color) -> VARCHAR`

The CSS keyword whose sRGB bytes exactly match the color, or NULL when no keyword matches. It is
an exact-match lookup, not a nearest-name search — most colors have no name:

```sql {"type":"duckfn","show":"table"}
SELECT plt_css_name(plt_srgb(1, 0, 0)) AS red,
       plt_css_name(plt_named('rebeccapurple')) AS purple,
       plt_css_name(plt_srgb(0.2, 0.4, 0.6)) IS NULL AS unnamed;
```

### `plt_css_formats()`

**Signature**: `plt_css_formats() -> VARCHAR[]`

The eight format names accepted by `plt_css_serialize`.

## Named colors

### `plt_named(name)`

**Signature**: `plt_named(name VARCHAR) -> plt_color`

One of palette's 148 SVG/CSS3 keyword colors, as an opaque sRGB color. Matching is case-insensitive.
An unknown name returns **NULL**:

```sql {"type":"duckfn","show":"table"}
SELECT plt_to_hex(plt_named('rebeccapurple')) AS purple,
       plt_to_hex(plt_named('Gold')) AS gold,
       plt_named('notacolor') IS NULL AS unknown_is_null;
```

### `plt_named_names()`

**Signature**: `plt_named_names() -> VARCHAR[]`

Every keyword name, sorted alphabetically. Use it to build a reference table without copying the
list into your SQL:

```sql {"type":"duckfn","show":"table"}
SELECT list_count(plt_named_names()) AS count,
       plt_named_names() AS names;
```

```sql {"type":"duckfn","show":"table"}
SELECT name, plt_to_hex(plt_named(name)) AS hex
FROM unnest(plt_named_names()) t(name)
WHERE name IN ('red', 'green', 'blue', 'rebeccapurple');
```

## Which function do I want?

| Input or output | Function |
| --- | --- |
| A hex string from a database column | `plt_from_hex` |
| A CSS color from a stylesheet or API | `plt_css_parse` |
| A short hex string for display | `plt_to_hex` |
| A specific CSS function, e.g. `oklch(...)` | `plt_css_serialize` |
| One of the 148 keyword colors | `plt_named` / `plt_named_names` |
| The keyword name of a color | `plt_css_name` |
| Whether text is a valid CSS color | `plt_css_is_valid` |
