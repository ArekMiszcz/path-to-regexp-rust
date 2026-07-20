use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::sync::LazyLock;

const DEFAULT_DELIMITER: char = '/';
static PATH_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\\.)|(?::(\w+)(?:\(((?:\\.|[^\\()])+)\))?|\(((?:\\.|[^\\()])+)\))([+*?])?")
        .expect("the token parser regex must be valid")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub delimiter: char,
    pub whitelist: Vec<String>,
    pub strict: bool,
    pub sensitive: bool,
    pub end: bool,
    pub start: bool,
    pub ends_with: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            delimiter: DEFAULT_DELIMITER,
            whitelist: Vec::new(),
            strict: false,
            sensitive: false,
            end: true,
            start: true,
            ends_with: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub name: String,
    pub prefix: String,
    pub delimiter: char,
    pub optional: bool,
    pub repeat: bool,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub token: Option<Token>,
    pub path: String,
}

fn escape_string(value: &str) -> String {
    regex::escape(value)
}

fn escape_group(group: &str) -> String {
    group
        .chars()
        .flat_map(|character| {
            if matches!(character, '=' | '!' | ':' | '$' | '/' | '(' | ')') {
                vec!['\\', character]
            } else {
                vec![character]
            }
        })
        .collect()
}

fn flags(route: &str, options: &Options) -> String {
    if !options.sensitive {
        format!("(?i){}", route)
    } else {
        route.to_owned()
    }
}

pub fn parse(text: &str, options: Options) -> Vec<Container> {
    let default_delimiter = options.delimiter;
    let mut index = 0;
    let mut key = 0;
    let mut path = String::new();
    let mut path_escaped = false;
    let mut containers = Vec::new();

    for captures in PATH_REGEXP.captures_iter(text) {
        let full_match = captures.get(0).expect("capture zero must exist");
        let offset = full_match.start();

        path.push_str(&text[index..offset]);
        index = full_match.end();

        if let Some(escaped) = captures.get(1) {
            path.push_str(&escaped.as_str()[1..]);
            path_escaped = true;
            continue;
        }

        let mut prefix = String::new();
        let name = captures.get(2).map_or("", |value| value.as_str());
        let capture = captures.get(3).map_or("", |value| value.as_str());
        let group = captures.get(4).map_or("", |value| value.as_str());
        let modifier = captures.get(5).map_or("", |value| value.as_str());

        if !path_escaped && !path.is_empty() {
            let candidate = path.chars().next_back().expect("path must not be empty");
            let accepted = options.whitelist.is_empty()
                || options
                    .whitelist
                    .iter()
                    .any(|allowed| allowed == &candidate.to_string());

            if accepted {
                prefix.push(candidate);
                path.pop();
            }
        }

        if !path.is_empty() {
            containers.push(Container {
                token: None,
                path: std::mem::take(&mut path),
            });
            path_escaped = false;
        }

        let repeat = modifier == "+" || modifier == "*";
        let optional = modifier == "?" || modifier == "*";
        let pattern = if !capture.is_empty() {
            capture
        } else if !group.is_empty() {
            group
        } else {
            ""
        };
        let delimiter = if prefix.is_empty() {
            default_delimiter
        } else {
            prefix.chars().next().expect("prefix must not be empty")
        };
        let name = if name.is_empty() {
            let generated = key.to_string();
            key += 1;
            generated
        } else {
            name.to_owned()
        };
        let pattern = if pattern.is_empty() {
            let delimiters = if delimiter == default_delimiter {
                delimiter.to_string()
            } else {
                format!("{delimiter}{default_delimiter}")
            };
            format!("[^{}]+?", escape_string(&delimiters))
        } else {
            escape_group(pattern)
        };

        containers.push(Container {
            token: Some(Token {
                name,
                prefix,
                delimiter,
                optional,
                repeat,
                pattern,
            }),
            path: String::new(),
        });
    }

    if !path.is_empty() || index < text.len() {
        path.push_str(&text[index..]);
        containers.push(Container { token: None, path });
    }

    containers
}

pub fn try_to_regexp(
    containers: &[Container],
    options: Options,
) -> fancy_regex::Result<FancyRegex> {
    let mut ends_with: Vec<String> = options
        .ends_with
        .iter()
        .map(|value| escape_string(value))
        .collect();
    ends_with.push("$".to_owned());
    let ends_with = ends_with.join("|");
    let mut route = if options.start {
        "^".to_owned()
    } else {
        String::new()
    };

    for container in containers {
        if !container.path.is_empty() {
            route.push_str(&escape_string(&container.path));
        } else if let Some(token) = &container.token {
            let capture = if token.repeat {
                format!(
                    "(?:{})(?:{}(?:{}))*",
                    token.pattern,
                    escape_string(&token.delimiter.to_string()),
                    token.pattern
                )
            } else {
                token.pattern.clone()
            };

            if token.optional {
                if token.prefix.is_empty() {
                    route.push_str(&format!("({capture})?"));
                } else {
                    route.push_str(&format!("(?:{}({capture}))?", escape_string(&token.prefix)));
                }
            } else {
                route.push_str(&format!("{}({capture})", escape_string(&token.prefix)));
            }
        }
    }

    if options.end {
        if !options.strict {
            route.push_str(&format!(
                "(?:{})?",
                escape_string(&options.delimiter.to_string())
            ));
        }

        if ends_with == "$" {
            route.push('$');
        } else {
            route.push_str(&format!("(?={ends_with})"));
        }
    } else {
        let is_end_delimited = containers.last().is_none_or(|container| {
            !container.path.is_empty() && container.path.ends_with(options.delimiter)
        });

        if !options.strict {
            route.push_str(&format!(
                "(?:{}(?={ends_with}))?",
                escape_string(&options.delimiter.to_string())
            ));
        }

        if !is_end_delimited {
            route.push_str(&format!(
                "(?={}|{ends_with})",
                escape_string(&options.delimiter.to_string())
            ));
        }
    }

    FancyRegex::new(&flags(&route, &options))
}

pub fn to_regexp(containers: &[Container], options: Options) -> FancyRegex {
    try_to_regexp(containers, options).expect("generated route regex must be valid")
}

pub fn try_match_str(
    text: &str,
    regexp: &FancyRegex,
    containers: &[Container],
) -> fancy_regex::Result<Vec<Match>> {
    let Some(captures) = regexp.captures(text)? else {
        return Ok(Vec::new());
    };
    let tokens = containers
        .iter()
        .filter_map(|container| container.token.as_ref());
    let matches = tokens
        .enumerate()
        .filter_map(|(index, token)| {
            captures.get(index + 1).map(|capture| Match {
                name: token.name.clone(),
                value: capture.as_str().to_owned(),
            })
        })
        .collect();

    Ok(matches)
}

pub fn match_str(text: &str, regexp: FancyRegex, containers: Vec<Container>) -> Vec<Match> {
    try_match_str(text, &regexp, &containers).expect("route matching must not exceed regex limits")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(route: &str, path: &str, options: Options) -> Vec<Match> {
        let containers = parse(route, options.clone());
        let regexp = to_regexp(&containers, options);
        match_str(path, regexp, containers)
    }

    fn is_match(route: &str, path: &str, options: Options) -> bool {
        let containers = parse(route, options.clone());
        to_regexp(&containers, options).is_match(path).unwrap()
    }

    #[test]
    fn matches_named_parameters() {
        assert_eq!(
            matches("/route/:foo/:id", "/route/john/7", Options::default()),
            vec![
                Match {
                    name: "foo".to_owned(),
                    value: "john".to_owned(),
                },
                Match {
                    name: "id".to_owned(),
                    value: "7".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn matches_static_routes() {
        assert!(is_match("/health", "/health", Options::default()));
        assert!(!is_match("/health", "/other", Options::default()));
    }

    #[test]
    fn makes_prefix_and_parameter_optional_together() {
        assert!(is_match("/user/:id?", "/user", Options::default()));
        assert!(is_match("/user/:id?", "/user/42", Options::default()));
        assert!(is_match("/user/:id?", "/user/", Options::default()));
    }

    #[test]
    fn supports_repeated_parameters() {
        assert_eq!(
            matches("/files/:path+", "/files/a/b/c", Options::default()),
            vec![Match {
                name: "path".to_owned(),
                value: "a/b/c".to_owned(),
            }]
        );
        assert!(is_match("/files/:path*", "/files", Options::default()));
    }

    #[test]
    fn supports_custom_patterns_and_unnamed_parameters() {
        assert!(is_match("/user/:id(\\d+)", "/user/42", Options::default()));
        assert!(!is_match(
            "/user/:id(\\d+)",
            "/user/nope",
            Options::default()
        ));
        assert_eq!(matches("/(\\d+)", "/42", Options::default())[0].name, "0");
    }

    #[test]
    fn treats_escaped_parameter_markers_as_literals() {
        assert!(is_match(r"/user/\:id", "/user/:id", Options::default()));
    }

    #[test]
    fn honors_case_sensitivity() {
        assert!(is_match("/User", "/user", Options::default()));
        let options = Options {
            sensitive: true,
            ..Options::default()
        };
        assert!(!is_match("/User", "/user", options));
    }

    #[test]
    fn honors_strict_trailing_delimiters() {
        assert!(is_match("/user", "/user/", Options::default()));
        let options = Options {
            strict: true,
            ..Options::default()
        };
        assert!(!is_match("/user", "/user/", options));
    }

    #[test]
    fn supports_non_ending_matches_at_segment_boundaries() {
        let options = Options {
            end: false,
            ..Options::default()
        };
        assert!(is_match("/user", "/user/42", options.clone()));
        assert!(!is_match("/user", "/username", options));
    }

    #[test]
    fn supports_unanchored_matches() {
        let options = Options {
            start: false,
            ..Options::default()
        };
        assert!(is_match("/user/:id", "/api/user/42", options));
    }

    #[test]
    fn supports_custom_end_markers() {
        let options = Options {
            ends_with: vec!["?".to_owned()],
            ..Options::default()
        };
        assert!(is_match("/user/:id", "/user/42?query", options));
    }

    #[test]
    fn handles_unicode_paths() {
        assert_eq!(
            matches("/café/:name", "/café/żółw", Options::default())[0].value,
            "żółw"
        );
    }

    #[test]
    fn whitelist_controls_parameter_prefixes() {
        let options = Options {
            whitelist: vec!["/".to_owned()],
            ..Options::default()
        };
        assert!(is_match("/file-:ext?", "/file-", options.clone()));
        assert!(!is_match("/file-:ext?", "/file", options));
    }

    #[test]
    fn omits_unmatched_optional_parameters() {
        assert!(matches("/user/:id?", "/user", Options::default()).is_empty());
    }

    #[test]
    fn captures_a_parameter_that_spans_the_entire_match() {
        assert_eq!(
            matches(":id", "42", Options::default()),
            vec![Match {
                name: "id".to_owned(),
                value: "42".to_owned(),
            }]
        );
    }
}
