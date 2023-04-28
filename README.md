<h1 align="center">CUE-basic</h1>
<hr>

CUE-basic is CUE, minus schema-checking, and computation.
You're left with a small but pleasant-to-write data format,
somewhere between JSON and CUE.

## Background

JSON is a wonderfully readable data format.  Things are explicit (all strings
are quoted, all items are comma-terminated, etc.); the structure of the text
mirrors the structure of the data - you can see the "shape" of the data just
by looking at the source text; and there are very few ways to writing any
given document (the only really wobbly part is that an object's keys can be
in any order you like).

JSON is not so pleasant to write, though.  There have been many attempts
to create formats with a JSON-equivalent data model, but with a focus on
ease-of-writing.  [JSON5] and [Hjson] are two examples.  These are very
similar to JSON, but more relaxed about punctuation: they permit unquoted
strings, and items can be terminated with a newline instead of a comma
(Hjson only).  They also add some new features like comments, multiline
strings, and hexadecimal numbers (JSON5 only).

[Hjson]: https://hjson.github.io/
[JSON5]: https://json5.org/

Probably the most famous write-focussed JSON-like format is [YAML].  Like JSON5
and Hjson, YAML is an extension of JSON (meaning that any valid JSON document
is also a valid YAML document); and it also likewise permits unquoted strings
and newline-terminated items, and adds in comments and multiline strings.
It departs even further from JSON by allowing you to define nesting using
indentation level, instead of braces.  It maintains JSON's "homomorphic"
property, whereby the data and source have the same shape.

[YAML]: https://yaml.org/

CUE-basic is another entry into this category.  Again, it's an extension of
JSON, so all valid JSON documents are also valid CUE-basic.  It's a subset of
the (much more powerful) CUE language: that means that all valid CUE-basic
documents are also valid CUE programs (and evaluate to the exact same JSON),

## Sample

```cue
// The outermost curly braces are optional

"package": {
    "name": "foomatic", // Looks like JSON so far...
    "version": "0.1.0"  // You can terminate with a newline instead of a comma
    edition: "2021"     // and you don't need to quote keys
}

dependencies: [
    anyhow: "1.0.70",
    clap: { version: "4.2.4", features: ["derive"] },
    itertools: "0.10.5", // Trailing commas are allowed
]

// You can go back and add keys to objects you've already defined:
package: {
    author: "Joe Bloggs" // This will be merged into "package"
}

profile: { debug: true }  // Instead of this...
profile: debug: true      // ...you can write this
profile: lto: "thin"
profile: codegen-units: 1 // Again, these will all be merged together

build: max_memory: 6Gi      // We have fancy numbers
build: threads: 8K          // Go crazy!
build: checksum: 0x25f93cd2 // and hex

// You can even do weird stuff like this:
locales: [{lang:    "en"}, {lang:    "jp"}, {lang:    "kr"}]
locales: [{country: "uk"}, {country: "ja"}, {country: "ko"}]
```
