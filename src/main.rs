extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate regex;

use regex::Regex;

/**
 * Default configs.
 */
const DEFAULT_DELIMITER: char = '/';

struct Options {
    delimiter: char,
    whitelist: Vec<String>
}
impl Default for Options {
    fn default () -> Options {
        Options {
            delimiter: DEFAULT_DELIMITER,
            whitelist: Vec::new()
        }
    }
}

#[derive(Debug)]
struct Token {
    name: String,
    prefix: String,
    delimiter: char,
    optional: bool,
    repeat: bool,
    pattern: String
}

/**
 * Escape a regular expression string.
 *
 * @param  {String} string
 * @return {String}
 */
fn escape_string (string: String) -> String {
    let re = Regex::new(r"([.+*?=^!:${}()[\]|/\\]])").unwrap();
    re.replace_all(string.as_str(), r"\$1").into_owned()
}

/**
 * Escape the capturing group by escaping special characters and meaning.
 *
 * @param  {String} group
 * @return {String}
 */
fn escape_group (group: String) -> String {
    let re = Regex::new(r"([=!:$/()])").unwrap();
    re.replace_all(group.as_str(), r"\$1").into_owned()
}

/**
 * Parse a string for the raw tokens.
 *
 * @param  {&str} text
 * @param  {Options} options
 * @return (Vec<String>, Vec<Token>)
 */
fn parse (text: &str, options: Options) -> (Vec<String>, Vec<Token>) {
    let default_delimiter: char = options.delimiter;
    let whitelist: &Vec<String> = &options.whitelist;
    let path_regexp: Regex = Regex::new(vec![
        // Match escaped characters that would otherwise appear in future matches.
        // This allows the user to escape special characters that won't transform.
        r"(\\.)",
        // Match Express-style parameters and un-named parameters with a prefix
        // and optional suffixes. Matches appear as:
        //
        // ":test(\\d+)?" => ["test", "\d+", NONE, "?"]
        // "(\\d+)"  => [NONE, NONE, "\d+", NONE]
        r"(?::(\w+)(?:\(((?:\\.|[^\\()])+)\))?|\(((?:\\.|[^\\()])+)\))([+*?])?"
    ].join("|").as_str()).unwrap();
    let mut index = 0;
    let mut key = -1;
    let mut path = String::new();
    let mut path_escaped = false;
    let mut paths: Vec<String> = vec![];
    let mut tokens: Vec<Token> = vec![];

    fn unwrap_match_to_str (m: Option<regex::Match>) -> &str {
        if m != None {
            m.unwrap().as_str()
        } else {
            ""
        }
    }

    debug!("{}", text);
    debug!("{}", path_regexp.is_match(text));

    if !path_regexp.is_match(text) {
        return (paths, tokens);
    }

    for res in path_regexp.captures_iter(text) {
        debug!("res {:#?}", res);

        let m = res.get(0).unwrap();
        let escaped = res.get(1);
        let offset = m.start();

        path.push_str(text.get(index..offset).unwrap());
        index = offset + m.as_str().len();

        // Ignore already escaped sequences.
        if escaped != None {
            path.push_str(escaped.unwrap().as_str());
            path_escaped = true;
            continue;
        }

        debug!("match {:#?}", m.as_str());
        debug!("escaped {:#?}", escaped);
        debug!("offset {:#?}", offset);
        debug!("path {:#?}", path);

        let mut prev: String = String::new();
        let name = unwrap_match_to_str(res.get(2));
        let capture = unwrap_match_to_str(res.get(3));
        let group = res.get(4);
        let modifier = unwrap_match_to_str(res.get(5));

        if !path_escaped && path.len() > 0 {
            let k = path.len();
            let c = String::from(path.get(k-1..k).unwrap());

            debug!("k {:#?}", k);
            debug!("c {:#?}", c);

            let matches: bool = if whitelist.len() > 0 {
                whitelist.into_iter().find(|&x| x == &c) != None
            } else {
                true
            };

            if matches {
                prev = c;
                path = String::from(path.get(0..k).unwrap());
            }
        }

        // Push the current path onto the tokens.
        if path != "" {
            paths.push(path);
            path = String::new();
            path_escaped = false;
        }

        let repeat = modifier == "+" || modifier == "*";
        let optional = modifier == "?" || modifier == "*";
        let pattern = if capture.len() > 0 {
            capture
        } else if group != None {
            group.unwrap().as_str()
        } else {
            ""
        };
        let delimiter: char = if prev != "" {
            prev.chars().next().unwrap()
        } else {
            default_delimiter
        };

        tokens.push(Token {
            name: if name != "" {
                name.to_owned()
            } else {
                key += 1;
                key.to_string()
            },
            prefix: prev,
            delimiter: delimiter,
            optional: optional,
            repeat: repeat,
            pattern: if pattern != "" {
                escape_group(pattern.to_owned())
            } else {
                let mut _pattern = String::from("[^");
                _pattern.push_str(escape_string(if delimiter == default_delimiter {
                    delimiter.to_string()
                } else {
                    vec![delimiter, default_delimiter].into_iter().collect()
                }).as_str());
                _pattern.push_str("]+?");
                _pattern
            }
        });

        debug!("tokens {:#?}", tokens);
    }

    // Push any remaining characters.
    if path.len() > 0 || index < text.len() {
        path.push_str(text.get(index..text.len()).unwrap());
        paths.push(path);
    }

    (paths, tokens)
}

fn main() {
    pretty_env_logger::init();

    let _s: &str = "/route/:foo/(.*)";
    let (paths, tokens) = parse(_s, Options::default());

    println!("paths {:#?}", paths);
    println!("tokens {:#?}", tokens);
}