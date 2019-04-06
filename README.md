# path_to_regexp
Turn a path string such as `/user/:name` into a regular expression

---

## Usage example
```rs
extern crate path_to_regexp;

use path_to_regexp::*;

fn main() {
    let scheme: &str = "/route/:foo/:bar/:id";
    let uri: &str = "/route/john/doe/7";
    
    let containers = parse(scheme, Options::default());
    let regexp = to_regexp(containers.as_ref(), Options::default());
    let matches = match_str(uri, regexp, containers.to_vec());

    println!("matches {:#?}", matches);

    /* OUTPUT:
    *    matches [
    *        Match {
    *            name: "foo",
    *            value: "john"
    *        },
    *        Match {
    *            name: "bar",
    *            value: "doe"
    *        },
    *        Match {
    *            name: "id",
    *            value: "7"
    *        }
    *    ]
    */
}
```

## Credit

This package is heavily inspired by its JavaScript
[path-to-regexp][path-to-regexp-js].