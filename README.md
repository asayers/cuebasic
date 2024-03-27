<h1 align="center">CUE-basic</h1>

CUE-basic makes JSON easier to write.
It's an extension of JSON, so every valid JSON document is also a valid CUE-basic document.
The data model is exactly the same as JSON, so a CUE-basic document can be converted into JSON.
The syntax is more flexible than JSON, which makes it more pleasant to write.

The syntax is copied from the [CUE language].
CUE-basic is a subset of CUE, which means that every valid CUE-basic document is also a valid CUE program.
Whereas CUE is a fully-fledged programming language, CUE-basic is just a data format.
This means that, compared to CUE, it's simple to define, and simple to process.

[CUE language]: https://cuelang.org/

## Sample

```cue
// If the file defines an object, the outermost curly braces are optional

"package": {
    "name": "foomatic",
    "version": "0.1.0"  // If your keys are on separate lines, you can omit the
                        // final comma
    edition: "2021"     // ...and you don't need to quote the key.  (You do have
                        // to quote the value though - we learned that lesson
                        // from YAML!)
    authors: [
        "Joe Bloggs",   // Lists do require commas to separate the values
        "Jane Doe",     // Trailing commas are allowed though!
    ],                  // ...in both lists and objects
}

dependencies: {
    anyhow: "1.0.70",
    clap: { version: "4.2.4", features: ["derive"] },
    itertools: "0.10.5",
}

// We have fancy numbers:
build: {
    theads: 1_400          // You can break them up with underscores
    timeout_seconds: 21.6K // and use SI suffixes
    max_memory: 6Gi        // ...in either base-10 or base-2
    checksum: 0x25f93cd2   // hexadecimal (octal and binary are supported too)
    precision: 1e-3        // and scientific notation, of course (this is
                           // valid in JSON too)
}

// You can go back and add keys to objects you've already defined:
package: {
    licence: "MIT"  // This will be merged into the "package" object defined above
    edition: "2021" // This is allowed, because it's the same as the
                    // previously-defined value
}

// In fact, a CUE-basic file really just defines a mapping from JSON paths to
// JSON values.
profile: { debug: true }    // So instead of this...
profile: debug: true        // ...you can write this
profile: lto: "thin"        // Again, these will all be merged together
profile: "codegen-units": 1 // Note that keys with special characters do need to
                            // be quoted

// You can even do weird stuff like this:
locales: [{lang:    "en"}, {lang:    "jp"}, {lang:    "kr"}]
locales: [{country: "uk"}, {country: "ja"}, {country: "ko"}]
```

Running this through cuebasic produces the following:

```json
{
  "build": {
    "checksum": 637091026,
    "max_memory": 6442450944,
    "precision": 0.001,
    "theads": 1400,
    "timeout_seconds": 21600
  },
  "dependencies": {
    "anyhow": "1.0.70",
    "clap": {
      "features": [ "derive" ],
      "version": "4.2.4"
    },
    "itertools": "0.10.5"
  },
  "locales": [
    { "country": "uk", "lang": "en" },
    { "country": "ja", "lang": "jp" },
    { "country": "ko", "lang": "kr" }
  ],
  "package": {
    "authors": [ "Joe Bloggs", "Jane Doe" ],
    "edition": "2021",
    "licence": "MIT",
    "name": "foomatic",
    "version": "0.1.0"
  },
  "profile": {
    "codegen-units": 1,
    "debug": true,
    "lto": "thin"
  }
}
```

## Background

There's a lot to like about JSON:

* It's very explicit: all strings are quoted, all items are comma-terminated,
  etc.
* The structure of the text mirrors the structure of the data - you can see the
  "shape" of the data just by looking at the source text.  (Sometimes people
  call this property "homomorphism".)
* There are very few ways to writing any given document (the only really wobbly
  part is that an object's keys can be in any order you like).

These properties make it an exceptionally _readable_ data format; JSON is not so
pleasant to write, though. There have been many attempts to create formats with
a JSON-equivalent data model, but with a focus on ease-of-writing.

Two minimalistic examples are [JSON5] and [Hjson].  These formats are very
similar to JSON, but more relaxed about punctuation: they permit unquoted
strings, and items can be terminated with a newline instead of a comma (Hjson
only).  They also add some new features like comments, multiline strings, and
hexadecimal numbers (JSON5 only).

[Hjson]: https://hjson.github.io/
[JSON5]: https://json5.org/

Probably the most famous writer-oriented JSON-like format is [YAML]. Like JSON5
and Hjson, YAML is an extension of JSON (meaning that any valid JSON document is
also a valid YAML document); and it also likewise permits unquoted strings and
newline-terminated items, and adds in comments and multiline strings. It departs
even further from JSON by allowing you to define nesting using indentation
level, instead of braces.  (Note that it has some [sharp corners] though.)

[YAML]: https://yaml.org/
[sharp corners]: https://ruudvanasseldonk.com/2023/01/11/the-yaml-document-from-hell

CUE-basic is another entry into this category.  Again, it's an extension of
JSON, so all valid JSON documents are also valid CUE-basic.  It's a subset
of the (much more powerful) CUE language: that means that all valid CUE-basic
documents are also valid CUE programs (and evaluate to the exact same JSON),

## Differences from CUE

* Expressions are not evaluated, so you can't write `x: 1 + 1`
* References are not allowed, so you can't write `x: y * 2`
* There's no unification or disjunction, so you can't use `&` or `|`
* Schema validation is gone, so you can't write `x: int`
* There's no support for string interpolation, so `"hello \(name)"` doesn't work
* The module system doesn't exist, so `package` and `import` don't work
* There's no support for multi-line strings - yet!  This is on the to-do list.

Basically, all the smart stuff that makes CUE powerful is gone, leaving a format
which is _almost_ as simple and boring as plain ol' JSON - but not quite.

# Implementation status

Well, it works!  And it's pretty fast.  The error messages are atrocious, however.

The "superset of JSON" claim is checked by fuzzing, so you can be fairly sure
that plain JSON will roundtrip through `cuebasic` unmodified.

The "subset of CUE" claim has **not** been systematically tested.  This should
be the next step: generating random CUE-basic documents and checking that
they're evaluated to the exact same JSON by both `cue` and `cuebasic`.
