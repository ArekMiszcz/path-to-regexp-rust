extern crate regex;
extern crate fancy_regex;

use regex::Regex;
use fancy_regex::Regex as FancyRegex;

/**
 * Default configs.
 */
const DEFAULT_DELIMITER: char = '/';

pub struct Options {
    delimiter: char,
    whitelist: Vec<String>,
    strict: bool,
    sensitive: bool,
    end: bool,
    start: bool,
    ends_with: Vec<String>
}
impl Default for Options {
    fn default () -> Options {
        Options {
            delimiter: DEFAULT_DELIMITER,
            whitelist: Vec::new(),
            strict: false,
            sensitive: false,
            end: true,
            start: true,
            ends_with: Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    name: String,
    prefix: String,
    delimiter: char,
    optional: bool,
    repeat: bool,
    pattern: String
}

#[derive(Debug)]
pub struct Match {
    name: String,
    value: String
}

#[derive(Debug, Clone)]
pub struct Container {
    token: Option<Token>,
    path: String
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
 * Get the flags for a regexp from the options.
 *
 * @param  {Options} options
 * @return {String}
 */
fn flags (route: &str, options: Options) -> String {
    if !options.sensitive {
        format!("(?i){}", route)
    } else {
        String::from(route)
    }
}

/**
 * Parse a string for the raw tokens and paths.
 *
 * @param  {&str} text
 * @param  {Options} options
 * @return (Vec<Container>)
 */
pub fn parse (text: &str, options: Options) -> Vec<Container> {
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
    let mut containers: Vec<Container> = vec![];

    fn unwrap_match_to_str (m: Option<regex::Match>) -> &str {
        if m != None {
            m.unwrap().as_str()
        } else {
            ""
        }
    }

    if !path_regexp.is_match(text) {
        return containers;
    }

    for res in path_regexp.captures_iter(text) {
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

        let mut prev: String = String::new();
        let name = unwrap_match_to_str(res.get(2));
        let capture = unwrap_match_to_str(res.get(3));
        let group = res.get(4);
        let modifier = unwrap_match_to_str(res.get(5));

        if !path_escaped && path.len() > 0 {
            let k = path.len();
            let c = String::from(path.get(k-1..k).unwrap());
            let matches: bool = if whitelist.len() > 0 {
                whitelist.into_iter().find(|&x| x == &c) != None
            } else {
                false
            };

            if matches {
                prev = c;
                path = String::from(path.get(0..k).unwrap());
            }
        }

        // Push the current path onto the tokens.
        if path != "" {
            containers.push(Container {
                path,
                token: None
            });
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

        containers.push(Container {
            path: String::new(),
            token: Some(Token {
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
                    let pattern_delimiter = if delimiter == default_delimiter {
                        delimiter.to_string()
                    } else {
                        vec![delimiter, default_delimiter].into_iter().collect()
                    };
                    
                    format!(r"[^\{}]+?", escape_string(pattern_delimiter))
                }
            })
        });
    }

    // Push any remaining characters.
    if path.len() > 0 || index < text.len() {
        path.push_str(text.get(index..text.len()).unwrap());
        containers.push(Container {
            path,
            token: None
        });
    }

    containers
}

/**
 * Expose a function for taking containers and returning a FancyRegex.
 *
 * @param  {Vec<Container>} containers
 * @param  {Options} options
 * @return {FancyRegex}
 */
pub fn to_regexp (containers: &Vec<Container>, options: Options) -> FancyRegex {
    let strict = options.strict;
    let start = options.start;
    let end = options.end;
    let delimiter = options.delimiter;
    let ends_with = if options.ends_with.len() > 0 {
        let mut _ends_with_vec = &options.ends_with;
        let mut _ends_with: Vec<String> = _ends_with_vec.into_iter().map(|s| {
            escape_string(s.to_string())
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

    // Iterate over the containers and create our regexp string.
    for i in 0..containers.len() {
        let container = &containers[i];
        
        if !container.path.is_empty() {
            route.push_str(escape_string(container.path.to_string()).as_str());
        } else {
            let token = container.token.as_ref().unwrap();
            let prefix = String::from(token.prefix.as_str());
            let capture = if token.repeat == true {
                format!("(?:{})(?:{}(?:{}))*", token.pattern.as_str(), escape_string(token.delimiter.to_string()).as_str(), token.pattern.as_str())
            } else {
                format!("{}", token.pattern.as_str())
            };

            if token.optional {
                if token.prefix != "" {
                    route.push_str(format!("({})", capture.as_str()).as_str());
                } else {
                    route.push_str(format!("(?:{}({}))?", escape_string(prefix).as_str(), capture.as_str()).as_str());
                }
            } else {
                route.push_str(format!("{}({})", escape_string(prefix).as_str(), capture.as_str()).as_str());
            }
        }
    }

    if end {
        if !strict {
            route.push_str(format!("(?:{})?", escape_string(delimiter.to_string())).as_str());
        }

        if ends_with == "$" {
            route.push_str("$");
        } else {
            route.push_str(format!("(?={})", ends_with).as_str());
        };
    } else {
        let last_container: &Container = containers.last().unwrap();
        let is_end_delimited = if !last_container.path.is_empty() {
            let last_path_char = last_container.path.get(last_container.path.len() - 1..last_container.path.len()).unwrap();
            last_path_char.to_string()
        } else {
            String::new()
        };

        if !strict {
            route.push_str(format!("(?:{}(?={}))?", escape_string(delimiter.to_string()), ends_with).as_str());
        }

        if is_end_delimited.is_empty() {
            route.push_str(format!("(?={}|{})", escape_string(delimiter.to_string()), ends_with).as_str());
        }
    }

    let regex_str = format!(r"{}", flags(route.as_str(), options).as_str());

    FancyRegex::new(regex_str.as_str()).unwrap()
}

/**
 * Function for matching text with parsed tokens.
 *
 * @param  {&str} text
 * @param  {FancyRegex} regexp
 * @param  {Vec<Container>} containers
 * @return {Vec<Match>}
 */
pub fn match_str (text: &str, regexp: FancyRegex, containers: Vec<Container>) -> Vec<Match> {
    let mut matches: Vec<Match> = vec![];

    if !regexp.is_match(text).unwrap() {
        return matches;
    }
    
    let containers: Vec<Container> = containers.into_iter()
        .filter(|container| container.path == "")
        .collect();

    if let Some(caps) = regexp.captures_from_pos(&text, 0).unwrap() {
        for i in 0..caps.len() {
            let cap = caps.at(i).unwrap();

            if cap.len() == text.len() {
                continue;
            }

            let container = containers.get(i-1).unwrap();
            if let Some(token) = &container.token {
                matches.push(Match {
                    name: String::from(token.name.as_str()),
                    value: cap.to_owned()
                });
            }
        }
    }

    matches
}