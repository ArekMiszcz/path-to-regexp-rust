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
    whitelist: Vec<String>,
    strict: bool,
    sensitive: bool,
    end: bool,
    start: bool,
    endsWith: Vec<String>
}
impl Default for Options {
    fn default () -> Options {
        Options {
            delimiter: DEFAULT_DELIMITER,
            whitelist: Vec::new(),
            strict: false,
            sensitive: false,
            end: false,
            start: false,
            endsWith: Vec::new()
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
    }

    // Push any remaining characters.
    if path.len() > 0 || index < text.len() {
        path.push_str(text.get(index..text.len()).unwrap());
        paths.push(path);
    }

    (paths, tokens)
}

/**
 * Pull out keys from a regexp.
 *
 * @param  {!RegExp} path
 * @param  {Array=}  keys
 * @return {!RegExp}
 */
fn regexpToRegexp (path: Regex, mut keys: Vec<Token>) -> Regex {
  if keys.len() <= 0 {
        return path;
  }

  // Use a negative lookahead to match only capturing groups.
  let groups = path.captures_iter(r"\((?!\?)");
  let count = groups.count();

  if count > 0 {
    for i in 0..count {
      keys.push(Token {
        name: i.to_string(),
        prefix: String::new(),
        delimiter: ' ',
        optional: false,
        repeat: false,
        pattern: String::new()
      });
    }
  }

  path
}

/**
 * Expose a function for taking tokens and returning a RegExp.
 *
 * @param  {!Array}  tokens
 * @param  {Array=}  keys
 * @param  {Object=} options
 * @return {!RegExp}
 */
// let endsWith = [].concat(options.endsWith || []).map(escape_string).concat('$').join('|');
fn tokensToRegExp (tokens: Vec<Token>, keys: Vec<Token>, options: Options) {
    // println!("paths {:#?}", paths);
    println!("tokens {:#?}", tokens);

    let strict = options.strict;
    let start = options.start;
    let end = options.end;
    let delimiter = options.delimiter;
    let ends_with = if options.endsWith.len() > 0 {
        let mut _ends_with_vec = options.endsWith;
        let mut _ends_with: Vec<String> = _ends_with_vec.into_iter().map(|s| {
            escape_string(s)
        }).collect();
        _ends_with.push(String::from("$"));
        _ends_with.join("|")
    } else {
        String::from("$")
    };
    let mut route = if start == true {
        String::from("^")
    } else {
        String::from("")
    };

    debug!("strict: {:#?}", strict);
    debug!("start: {:#?}", start);
    debug!("end: {:#?}", end);
    debug!("delimiter: {:#?}", delimiter);
    debug!("ends_with: {:#?}", ends_with);
    debug!("route: {:#?}", route);

    // Iterate over the tokens and create our regexp string.
    for i in 0..tokens.len() {
        let token = &tokens[i];
        let prefix = String::from(token.prefix.as_str());
        let capture = if token.repeat == true {
            format!("(?:{})(?:{}(?:{}))*", token.pattern.as_str(), escape_string(token.delimiter.to_string()).as_str(), token.pattern.as_str())
        } else {
            format!("{}", token.pattern.as_str())
        };
        
        debug!("capture: {:#?}", capture);

        if keys.len() > 0 {
            // keys.push(token);
        }

        if token.optional {
            if token.prefix != "" {
                route = format!("({})", capture.as_str());
            } else {
                route = format!("(?:{}({}))?", escape_string(prefix).as_str(), capture.as_str());
            }
        } else {
            route = format!("{}({})", escape_string(prefix).as_str(), capture.as_str());
        }

        debug!("route: {:#?}", route);
    }
}

fn main() {
    pretty_env_logger::init();

    let _s: &str = "/route/:foo/(.*)";
    let (paths, tokens) = parse(_s, Options::default());

    tokensToRegExp(tokens, vec![], Options::default());
}