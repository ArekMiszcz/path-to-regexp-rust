use path_to_regexp::{Container, Match, Options, Token, parse, to_regexp, try_match_str};

#[derive(Clone, Copy)]
struct Case<'a> {
    route: &'a str,
    path: &'a str,
    captures: Option<&'a [(&'a str, &'a str)]>,
}

fn assert_case(case: Case<'_>, options: Options) {
    let containers = parse(case.route, options.clone());
    let regexp = to_regexp(&containers, options);
    let actual_match = regexp.is_match(case.path).unwrap();

    assert_eq!(
        actual_match,
        case.captures.is_some(),
        "route {:?} against {:?}",
        case.route,
        case.path
    );

    if let Some(expected) = case.captures {
        let actual = try_match_str(case.path, &regexp, &containers).unwrap();
        let expected: Vec<Match> = expected
            .iter()
            .map(|(name, value)| Match {
                name: (*name).to_owned(),
                value: (*value).to_owned(),
            })
            .collect();
        assert_eq!(
            actual, expected,
            "route {:?} against {:?}",
            case.route, case.path
        );
    }
}

fn assert_cases(cases: &[Case<'_>], options: Options) {
    for case in cases {
        assert_case(*case, options.clone());
    }
}

#[test]
fn simple_paths_and_parameters_match_v3() {
    assert_cases(
        &[
            Case {
                route: "/",
                path: "/",
                captures: Some(&[]),
            },
            Case {
                route: "/",
                path: "/route",
                captures: None,
            },
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/route",
                captures: None,
            },
            Case {
                route: "/test",
                path: "/test/route",
                captures: None,
            },
            Case {
                route: "/test",
                path: "/test/",
                captures: Some(&[]),
            },
            Case {
                route: "/test/",
                path: "/test",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/test/",
                captures: Some(&[]),
            },
            Case {
                route: "/test/",
                path: "/test//",
                captures: Some(&[]),
            },
            Case {
                route: "test",
                path: "test",
                captures: Some(&[]),
            },
            Case {
                route: "test",
                path: "/test",
                captures: None,
            },
            Case {
                route: "/:test",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test",
                path: "/another",
                captures: Some(&[("test", "another")]),
            },
            Case {
                route: "/:test",
                path: "/something/else",
                captures: None,
            },
            Case {
                route: "/:test",
                path: "/route.json",
                captures: Some(&[("test", "route.json")]),
            },
            Case {
                route: "/:test",
                path: "/something%2Felse",
                captures: Some(&[("test", "something%2Felse")]),
            },
            Case {
                route: "/:test",
                path: "/;,:@&=+$-_.!~*()",
                captures: Some(&[("test", ";,:@&=+$-_.!~*()")]),
            },
            Case {
                route: ":test",
                path: "route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: ":test",
                path: "/route",
                captures: None,
            },
            Case {
                route: ":test",
                path: "route/",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:foo/:bar",
                path: "/match/route",
                captures: Some(&[("foo", "match"), ("bar", "route")]),
            },
            Case {
                route: "/:foo/:bar",
                path: "/match",
                captures: None,
            },
            Case {
                route: "/:foo/:bar",
                path: "/match/route/more",
                captures: None,
            },
            Case {
                route: "/café",
                path: "/café",
                captures: Some(&[]),
            },
            Case {
                route: "/café",
                path: "/cafe",
                captures: None,
            },
            Case {
                route: "/:foo",
                path: "/café",
                captures: Some(&[("foo", "café")]),
            },
            Case {
                route: "/:foo",
                path: "/żółw",
                captures: Some(&[("foo", "żółw")]),
            },
            Case {
                route: "packages/",
                path: "packages",
                captures: None,
            },
            Case {
                route: "packages/",
                path: "packages/",
                captures: Some(&[]),
            },
            Case {
                route: "/:test/",
                path: "/route",
                captures: None,
            },
            Case {
                route: "/:test/",
                path: "/route/",
                captures: Some(&[("test", "route")]),
            },
        ],
        Options::default(),
    );
}

#[test]
fn optional_and_repeated_parameters_match_v3() {
    assert_cases(
        &[
            Case {
                route: "/:test?",
                path: "",
                captures: Some(&[]),
            },
            Case {
                route: "/:test?",
                path: "/",
                captures: Some(&[]),
            },
            Case {
                route: "/:test?",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test?",
                path: "/route/nested",
                captures: None,
            },
            Case {
                route: "/:test?",
                path: "//",
                captures: None,
            },
            Case {
                route: "/:test?/bar",
                path: "/bar",
                captures: Some(&[]),
            },
            Case {
                route: "/:test?/bar",
                path: "/foo/bar",
                captures: Some(&[("test", "foo")]),
            },
            Case {
                route: "/:test?/bar",
                path: "//bar",
                captures: None,
            },
            Case {
                route: "/:test?-bar",
                path: "-bar",
                captures: Some(&[]),
            },
            Case {
                route: "/:test?-bar",
                path: "/-bar",
                captures: None,
            },
            Case {
                route: "/:test?-bar",
                path: "/foo-bar",
                captures: Some(&[("test", "foo")]),
            },
            Case {
                route: "/:test*-bar",
                path: "-bar",
                captures: Some(&[]),
            },
            Case {
                route: "/:test*-bar",
                path: "/foo-bar",
                captures: Some(&[("test", "foo")]),
            },
            Case {
                route: "/:test*-bar",
                path: "/foo/baz-bar",
                captures: Some(&[("test", "foo/baz")]),
            },
            Case {
                route: "/:test+",
                path: "/",
                captures: None,
            },
            Case {
                route: "/:test+",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test+",
                path: "/some/basic/route",
                captures: Some(&[("test", "some/basic/route")]),
            },
            Case {
                route: "/:test+",
                path: "//",
                captures: None,
            },
            Case {
                route: "/:test*",
                path: "",
                captures: Some(&[]),
            },
            Case {
                route: "/:test*",
                path: "/",
                captures: Some(&[]),
            },
            Case {
                route: "/:test*",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test*",
                path: "/some/basic/route",
                captures: Some(&[("test", "some/basic/route")]),
            },
            Case {
                route: "/:test*",
                path: "//",
                captures: None,
            },
            Case {
                route: ":test?",
                path: "",
                captures: Some(&[]),
            },
            Case {
                route: ":test?",
                path: "route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: ":test?",
                path: "/route",
                captures: None,
            },
            Case {
                route: ":test?",
                path: "route/foobar",
                captures: None,
            },
            Case {
                route: ":test+",
                path: "",
                captures: None,
            },
            Case {
                route: ":test+",
                path: "route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: ":test+",
                path: "foo/bar",
                captures: Some(&[("test", "foo/bar")]),
            },
            Case {
                route: "/:foo+baz",
                path: "/foobaz",
                captures: Some(&[("foo", "foo")]),
            },
            Case {
                route: "/:foo+baz",
                path: "/foo/barbaz",
                captures: Some(&[("foo", "foo/bar")]),
            },
            Case {
                route: "/:foo+baz",
                path: "/baz",
                captures: None,
            },
            Case {
                route: "/:foo?/:bar?-ext",
                path: "-ext",
                captures: Some(&[]),
            },
            Case {
                route: "/:foo?/:bar?-ext",
                path: "/foo-ext",
                captures: Some(&[("foo", "foo")]),
            },
            Case {
                route: "/:foo?/:bar?-ext",
                path: "/foo/bar-ext",
                captures: Some(&[("foo", "foo"), ("bar", "bar")]),
            },
            Case {
                route: "/:foo?/:bar?-ext",
                path: "/foo/-ext",
                captures: None,
            },
            Case {
                route: "/:required/:optional?-ext",
                path: "/foo-ext",
                captures: Some(&[("required", "foo")]),
            },
            Case {
                route: "/:required/:optional?-ext",
                path: "/foo/bar-ext",
                captures: Some(&[("required", "foo"), ("optional", "bar")]),
            },
            Case {
                route: "/:required/:optional?-ext",
                path: "/foo/-ext",
                captures: None,
            },
        ],
        Options::default(),
    );
}

#[test]
fn custom_patterns_and_formats_match_v3() {
    assert_cases(
        &[
            Case {
                route: "/:test(\\d+)",
                path: "/123",
                captures: Some(&[("test", "123")]),
            },
            Case {
                route: "/:test(\\d+)",
                path: "/abc",
                captures: None,
            },
            Case {
                route: "/:test(\\d+)",
                path: "/123/abc",
                captures: None,
            },
            Case {
                route: "/:test(.*)",
                path: "/",
                captures: Some(&[("test", "")]),
            },
            Case {
                route: "/:test(.*)",
                path: "/anything/goes/here",
                captures: Some(&[("test", "anything/goes/here")]),
            },
            Case {
                route: "/:test(foo/bar)",
                path: "/foo/bar",
                captures: Some(&[("test", "foo/bar")]),
            },
            Case {
                route: "/:route([a-z]+)",
                path: "/abcde",
                captures: Some(&[("route", "abcde")]),
            },
            Case {
                route: "/:route([a-z]+)",
                path: "/12345",
                captures: None,
            },
            Case {
                route: "/:route(this|that)",
                path: "/this",
                captures: Some(&[("route", "this")]),
            },
            Case {
                route: "/:route(this|that)",
                path: "/that",
                captures: Some(&[("route", "that")]),
            },
            Case {
                route: "/:route(this|that)",
                path: "/foo",
                captures: None,
            },
            Case {
                route: "/:path(abc|xyz)*",
                path: "",
                captures: Some(&[]),
            },
            Case {
                route: "/:path(abc|xyz)*",
                path: "/abc",
                captures: Some(&[("path", "abc")]),
            },
            Case {
                route: "/:path(abc|xyz)*",
                path: "/abc/xyz",
                captures: Some(&[("path", "abc/xyz")]),
            },
            Case {
                route: "/:path(abc|xyz)*",
                path: "/xyz/abc/xyz",
                captures: Some(&[("path", "xyz/abc/xyz")]),
            },
            Case {
                route: "/:path(abc|xyz)*",
                path: "/xyzxyz",
                captures: None,
            },
            Case {
                route: "/:test(\\d+)+",
                path: "/123/456/789",
                captures: Some(&[("test", "123/456/789")]),
            },
            Case {
                route: "/:test(\\d+)+",
                path: "/abc/456/789",
                captures: None,
            },
            Case {
                route: "/route.:ext(json|xml)+",
                path: "/route.json",
                captures: Some(&[("ext", "json")]),
            },
            Case {
                route: "/route.:ext(json|xml)+",
                path: "/route.xml.json",
                captures: Some(&[("ext", "xml.json")]),
            },
            Case {
                route: "/route.:ext(json|xml)+",
                path: "/route.html",
                captures: None,
            },
            Case {
                route: "/test.json",
                path: "/test.json",
                captures: Some(&[]),
            },
            Case {
                route: "/test.json",
                path: "/route.json",
                captures: None,
            },
            Case {
                route: "/:test.json",
                path: "/test.json",
                captures: Some(&[("test", "test")]),
            },
            Case {
                route: "/:test.json",
                path: "/route.json.json",
                captures: Some(&[("test", "route.json")]),
            },
            Case {
                route: "/test.:format",
                path: "/test.html",
                captures: Some(&[("format", "html")]),
            },
            Case {
                route: "/test.:format",
                path: "/test.hbs.html",
                captures: None,
            },
            Case {
                route: "/test.:format.:format",
                path: "/test.html",
                captures: None,
            },
            Case {
                route: "/test.:format.:format",
                path: "/test.hbs.html",
                captures: Some(&[("format", "hbs"), ("format", "html")]),
            },
            Case {
                route: "/test.:format+",
                path: "/test.html",
                captures: Some(&[("format", "html")]),
            },
            Case {
                route: "/test.:format+",
                path: "/test.hbs.html",
                captures: Some(&[("format", "hbs.html")]),
            },
            Case {
                route: "/test.:format.",
                path: "/test.html.",
                captures: Some(&[("format", "html")]),
            },
            Case {
                route: "/test.:format.",
                path: "/test.hbs.html",
                captures: None,
            },
            Case {
                route: "/:test.:format",
                path: "/route.html",
                captures: Some(&[("test", "route"), ("format", "html")]),
            },
            Case {
                route: "/:test.:format",
                path: "/route",
                captures: None,
            },
            Case {
                route: "/:test.:format",
                path: "/route.html.json",
                captures: Some(&[("test", "route.html"), ("format", "json")]),
            },
            Case {
                route: "/:test.:format?",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test.:format?",
                path: "/route.json",
                captures: Some(&[("test", "route"), ("format", "json")]),
            },
            Case {
                route: "/:test.:format?",
                path: "/route.json.html",
                captures: Some(&[("test", "route.json"), ("format", "html")]),
            },
            Case {
                route: "/(\\d+)",
                path: "/123",
                captures: Some(&[("0", "123")]),
            },
            Case {
                route: "/(\\d+)",
                path: "/abc",
                captures: None,
            },
            Case {
                route: "/(\\d+)?",
                path: "/",
                captures: Some(&[]),
            },
            Case {
                route: "/(\\d+)?",
                path: "/123",
                captures: Some(&[("0", "123")]),
            },
            Case {
                route: "/(.*)",
                path: "/",
                captures: Some(&[("0", "")]),
            },
            Case {
                route: "/(.*)",
                path: "/route/nested",
                captures: Some(&[("0", "route/nested")]),
            },
        ],
        Options::default(),
    );
}

#[test]
fn matching_options_match_v3() {
    let sensitive = Options {
        sensitive: true,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/TEST",
                captures: None,
            },
            Case {
                route: "/TEST",
                path: "/test",
                captures: None,
            },
            Case {
                route: "/TEST",
                path: "/TEST",
                captures: Some(&[]),
            },
            Case {
                route: "/:test(abc)",
                path: "/abc",
                captures: Some(&[("test", "abc")]),
            },
            Case {
                route: "/:test(abc)",
                path: "/ABC",
                captures: None,
            },
        ],
        sensitive,
    );

    let strict = Options {
        strict: true,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test/",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/test",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/test/",
                captures: Some(&[]),
            },
            Case {
                route: "/test/",
                path: "/test//",
                captures: None,
            },
            Case {
                route: "/:test",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test",
                path: "/route/",
                captures: None,
            },
            Case {
                route: "/:test?",
                path: "",
                captures: Some(&[]),
            },
            Case {
                route: "/:test?",
                path: "/",
                captures: None,
            },
            Case {
                route: "/:test?",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
        ],
        strict,
    );

    let end_false = Options {
        end: false,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test/",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test/route",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/testing",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/test",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/test/route",
                captures: Some(&[]),
            },
            Case {
                route: "/test/",
                path: "/test//",
                captures: Some(&[]),
            },
            Case {
                route: "/:test",
                path: "/route/next",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test(\\d+)",
                path: "/123/abc",
                captures: Some(&[("test", "123")]),
            },
            Case {
                route: "/test.json",
                path: "/test.json.hbs",
                captures: None,
            },
            Case {
                route: "/test.json",
                path: "/test.json/route",
                captures: Some(&[]),
            },
        ],
        end_false,
    );

    let start_false = Options {
        start: false,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/route/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test/route",
                captures: None,
            },
            Case {
                route: "/test",
                path: "/route/test/deep",
                captures: None,
            },
            Case {
                route: "/test/",
                path: "/route/test/",
                captures: Some(&[]),
            },
            Case {
                route: "/:test",
                path: "/route",
                captures: Some(&[("test", "route")]),
            },
            Case {
                route: "/:test",
                path: "/api/route",
                captures: Some(&[("test", "route")]),
            },
        ],
        start_false,
    );

    let combined = Options {
        start: false,
        end: false,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/route/test/deep",
                captures: Some(&[]),
            },
            Case {
                route: "/test/",
                path: "/route/test/deep",
                captures: Some(&[]),
            },
            Case {
                route: "/test.json",
                path: "/route/test.json/deep",
                captures: Some(&[]),
            },
            Case {
                route: "/:test",
                path: "/api/route/deep",
                captures: Some(&[("test", "api")]),
            },
        ],
        combined,
    );
}

#[test]
fn delimiters_whitelist_and_end_markers_match_v3() {
    let dot = Options {
        delimiter: '.',
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: ":domain.com",
                path: "example.com",
                captures: Some(&[("domain", "example")]),
            },
            Case {
                route: ":domain.com",
                path: "github.com",
                captures: Some(&[("domain", "github")]),
            },
            Case {
                route: "mail.:domain.com",
                path: "mail.example.com",
                captures: Some(&[("domain", "example")]),
            },
            Case {
                route: "mail.:domain.com",
                path: "mail.github.com",
                captures: Some(&[("domain", "github")]),
            },
            Case {
                route: "example.:ext",
                path: "example.com",
                captures: Some(&[("ext", "com")]),
            },
            Case {
                route: "example.:ext",
                path: "example.org",
                captures: Some(&[("ext", "org")]),
            },
        ],
        dot,
    );

    let dollar = Options {
        delimiter: '$',
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "$:foo$:bar?",
                path: "$x",
                captures: Some(&[("foo", "x")]),
            },
            Case {
                route: "$:foo$:bar?",
                path: "$x$y",
                captures: Some(&[("foo", "x"), ("bar", "y")]),
            },
            Case {
                route: "$:foo$:bar?",
                path: "$x$y$z",
                captures: None,
            },
        ],
        dollar,
    );

    let space = Options {
        delimiter: ' ',
        end: false,
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "this is",
                path: "this is a test",
                captures: Some(&[]),
            },
            Case {
                route: "this is",
                path: "this isn't",
                captures: None,
            },
            Case {
                route: ":test+",
                path: "hello",
                captures: Some(&[("test", "hello")]),
            },
            Case {
                route: ":test+",
                path: "hello world",
                captures: Some(&[("test", "hello world")]),
            },
        ],
        space,
    );

    let ends_with = Options {
        ends_with: vec!["?".to_owned()],
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "/test",
                path: "/test",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test?query=string",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/test/?query=string",
                captures: Some(&[]),
            },
            Case {
                route: "/test",
                path: "/testx",
                captures: None,
            },
        ],
        ends_with,
    );

    let whitelist = Options {
        whitelist: vec!["/".to_owned()],
        ..Options::default()
    };
    assert_cases(
        &[
            Case {
                route: "name/:attr1?-:attr2?",
                path: "name/1",
                captures: None,
            },
            Case {
                route: "name/:attr1?-:attr2?",
                path: "name/1-",
                captures: Some(&[("attr1", "1")]),
            },
            Case {
                route: "name/:attr1?-:attr2?",
                path: "name/1-2",
                captures: Some(&[("attr1", "1"), ("attr2", "2")]),
            },
            Case {
                route: "name/:attr1?-:attr2?",
                path: "name/1-2-3",
                captures: Some(&[("attr1", "1"), ("attr2", "2-3")]),
            },
        ],
        whitelist,
    );
}

#[test]
fn escaped_characters_match_v3() {
    assert_cases(
        &[
            Case {
                route: r"/\(testing\)",
                path: "/testing",
                captures: None,
            },
            Case {
                route: r"/\(testing\)",
                path: "/(testing)",
                captures: Some(&[]),
            },
            Case {
                route: r"/:foo\?",
                path: "/route?",
                captures: Some(&[("foo", "route")]),
            },
            Case {
                route: r"/:foo\?",
                path: "/route",
                captures: None,
            },
            Case {
                route: r"\/:pre?baz",
                path: "/foobaz",
                captures: Some(&[("pre", "foo")]),
            },
            Case {
                route: r"\/:pre?baz",
                path: "/baz",
                captures: Some(&[]),
            },
            Case {
                route: r"/:foo\(:bar?\)",
                path: "/hello(world)",
                captures: Some(&[("foo", "hello"), ("bar", "world")]),
            },
            Case {
                route: r"/:foo\(:bar?\)",
                path: "/hello()",
                captures: Some(&[("foo", "hello")]),
            },
            Case {
                route: r"/\:name",
                path: "/:name",
                captures: Some(&[]),
            },
            Case {
                route: r"/\.\+\*\?\=\^\!\:\$\[\]\|",
                path: "/.+*?=^!:$[]|",
                captures: Some(&[]),
            },
            Case {
                route: r"/test\/:uid(u\d+)?:cid(c\d+)?",
                path: "/test/",
                captures: Some(&[]),
            },
            Case {
                route: r"/test\/:uid(u\d+)?:cid(c\d+)?",
                path: "/test/u123",
                captures: Some(&[("uid", "u123")]),
            },
            Case {
                route: r"/test\/:uid(u\d+)?:cid(c\d+)?",
                path: "/test/c123",
                captures: Some(&[("cid", "c123")]),
            },
        ],
        Options::default(),
    );
}

#[test]
fn parse_outputs_match_v3_token_metadata() {
    assert_eq!(
        parse("/:test", Options::default()),
        vec![Container {
            token: Some(Token {
                name: "test".to_owned(),
                prefix: "/".to_owned(),
                delimiter: '/',
                optional: false,
                repeat: false,
                pattern: "[^/]+?".to_owned(),
            }),
            path: String::new(),
        }]
    );
    assert_eq!(
        parse("/route.:ext(json|xml)+", Options::default()),
        vec![
            Container {
                token: None,
                path: "/route".to_owned()
            },
            Container {
                token: Some(Token {
                    name: "ext".to_owned(),
                    prefix: ".".to_owned(),
                    delimiter: '.',
                    optional: false,
                    repeat: true,
                    pattern: "json|xml".to_owned(),
                }),
                path: String::new(),
            },
        ]
    );
    assert_eq!(
        parse("/(\\d+)?", Options::default()),
        vec![Container {
            token: Some(Token {
                name: "0".to_owned(),
                prefix: "/".to_owned(),
                delimiter: '/',
                optional: true,
                repeat: false,
                pattern: "\\d+".to_owned(),
            }),
            path: String::new(),
        }]
    );
    assert_eq!(
        parse(r"/\:test", Options::default()),
        vec![Container {
            token: None,
            path: "/:test".to_owned()
        }]
    );
    assert_eq!(
        parse("/static", Options::default()),
        vec![Container {
            token: None,
            path: "/static".to_owned()
        }]
    );
    assert!(parse("", Options::default()).is_empty());
}

#[test]
fn fallible_api_handles_non_matches_and_empty_containers() {
    let containers = parse("/user/:id", Options::default());
    let regexp = to_regexp(&containers, Options::default());
    assert!(
        try_match_str("/other", &regexp, &containers)
            .unwrap()
            .is_empty()
    );

    let containers = vec![Container {
        token: None,
        path: String::new(),
    }];
    let regexp = to_regexp(&containers, Options::default());
    assert!(regexp.is_match("").unwrap());
}
