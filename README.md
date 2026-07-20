# path_to_regexp
Turn a path string such as `/user/:name` into a regular expression.

## Features

- Named and unnamed parameters
- Optional (`?`) and repeated (`+`, `*`) parameters
- Custom parameter patterns
- Case-sensitive, strict, partial, and unanchored matching options

## Usage example
```rust
use path_to_regexp::*;

fn main() {
    let scheme = "/route/:foo/:bar/:id";
    let uri = "/route/john/doe/7";

    let containers = parse(scheme, Options::default());
    let regexp = to_regexp(&containers, Options::default());
    let matches = match_str(uri, regexp, containers);

    println!("matches {:#?}", matches);
}
```

`Options` and match fields are public, so behavior can be customized and captured values can be consumed directly. `try_to_regexp` and `try_match_str` are available when regex errors need to be handled instead of panicking.

## Credit

This package is heavily inspired by its JavaScript
[path-to-regexp][path-to-regexp-js].

[path-to-regexp-js]: https://github.com/pillarjs/path-to-regexp