use bevy::prelude::error;
use cssparser::{
    AtRuleParser, DeclarationListParser, DeclarationParser, ParseError, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, ToCss, Token,
};
use smallvec::{smallvec, SmallVec};

use crate::{
    property::PropertyValues,
    selector::{Selector, SelectorElement},
    stylesheet::StyleRule,
    EcssError,
};

pub(crate) struct StyleSheetParser;

impl StyleSheetParser {
    pub(crate) fn parse(content: &str) -> SmallVec<[StyleRule; 8]> {
        let mut input = ParserInput::new(content);
        let mut parser = Parser::new(&mut input);

        RuleListParser::new_for_stylesheet(&mut parser, StyleSheetParser)
            .into_iter()
            .filter_map(|result| match result {
                Ok(rule) => Some(rule),
                Err((err, rule)) => {
                    error!(
                        "Failed to parse rule: {}. Error: {}",
                        rule,
                        format_error(err)
                    );
                    None
                }
            })
            .collect()
    }
}

fn format_error<'i>(error: ParseError<'i, EcssError>) -> String {
    let error_description = match error.kind {
        cssparser::ParseErrorKind::Basic(b) => match b {
            cssparser::BasicParseErrorKind::UnexpectedToken(token) => {
                format!("Unexpected token {}", token.to_css_string())
            }
            cssparser::BasicParseErrorKind::EndOfInput => "End of input".to_string(),
            cssparser::BasicParseErrorKind::AtRuleInvalid(token) => {
                format!("At rule isn't supported {}", token.to_string())
            }
            cssparser::BasicParseErrorKind::AtRuleBodyInvalid => format!("At rule isn't supported"),
            cssparser::BasicParseErrorKind::QualifiedRuleInvalid => format!("Invalid rule"),
        },
        cssparser::ParseErrorKind::Custom(c) => c.to_string(),
    };

    format!(
        "{} at {}:{}",
        error_description, error.location.line, error.location.column
    )
}

impl<'i> QualifiedRuleParser<'i> for StyleSheetParser {
    type Prelude = Selector;

    type QualifiedRule = StyleRule;

    type Error = EcssError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Ok(parse_selector(input)?)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut rule = StyleRule {
            selector: prelude,
            tokens: Default::default(),
        };

        for property in DeclarationListParser::new(input, PropertyParser) {
            match property {
                Ok((name, property)) => {
                    rule.tokens.insert(name, property);
                }
                Err((err, a)) => println!("Failed: {:?} ({})", err, a),
            }
        }

        Ok(rule)
    }
}

fn parse_selector<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<Selector, ParseError<'i, EcssError>> {
    let mut elements = smallvec![];

    let mut next_is_class = false;

    while let Ok(token) = parser.next_including_whitespace() {
        use cssparser::Token::*;
        match token {
            Ident(v) => {
                if next_is_class {
                    next_is_class = false;
                    elements.push(SelectorElement::Class(v.to_string()));
                } else {
                    elements.push(SelectorElement::Component(v.to_string()));
                }
            }
            IDHash(v) => {
                if v.is_empty() {
                    return Err(parser.new_custom_error(EcssError::InvalidSelector));
                } else {
                    elements.push(SelectorElement::Name(v.to_string()));
                }
            }
            WhiteSpace(_) => elements.push(SelectorElement::Child),
            Delim(c) if *c == '.' => next_is_class = true,
            _ => {
                let token = token.to_css_string();
                return Err(parser.new_custom_error(EcssError::UnexpectedToken(token)));
            }
        }
    }

    if elements.is_empty() {
        return Err(parser.new_custom_error(EcssError::InvalidSelector));
    }

    // Remove noise the trailing white spaces, if any
    while elements.len() > 0 && elements.last().unwrap() == &SelectorElement::Child {
        elements.remove(elements.len() - 1);
    }

    Ok(Selector::new(elements))
}

impl<'i> AtRuleParser<'i> for StyleSheetParser {
    type Prelude = ();

    type AtRule = StyleRule;

    type Error = EcssError;
}

struct PropertyParser;

impl<'i> DeclarationParser<'i> for PropertyParser {
    type Declaration = (String, PropertyValues);

    type Error = EcssError;

    fn parse_value<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        parser: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, EcssError>> {
        let mut tokens = smallvec![];
        for token in parse_values(parser)? {
            match token.try_into() {
                Ok(t) => tokens.push(t),
                Err(_) => continue,
            }
        }

        Ok((name.to_string(), PropertyValues(tokens)))
    }
}

impl<'i> AtRuleParser<'i> for PropertyParser {
    type Prelude = ();
    type AtRule = (String, PropertyValues);
    type Error = EcssError;
}

fn parse_values<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<SmallVec<[Token<'i>; 8]>, ParseError<'i, EcssError>> {
    let mut values = SmallVec::new();

    while let Ok(token) = parser.next_including_whitespace() {
        values.push(token.clone())
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert!(
            StyleSheetParser::parse("").is_empty(),
            "Should return an empty list of rules"
        );
        assert!(
            StyleSheetParser::parse("{}").is_empty(),
            "\"{{}}\" Should return an empty list of rules"
        );
        assert!(
            StyleSheetParser::parse(" {}").is_empty(),
            "\" {{}}\" Should return an empty list of rules"
        );
        assert!(
            StyleSheetParser::parse("# {}").is_empty(),
            "\"# {{}}\" Should return an empty list of rules"
        );
        assert!(
            StyleSheetParser::parse("@@@ {}").is_empty(),
            "Should return an empty list of rules"
        );
        assert!(
            StyleSheetParser::parse("{}{}").is_empty(),
            "Should return an empty list of rules"
        );
    }

    #[test]
    fn parse_single_name_selector_no_property() {
        let rules = StyleSheetParser::parse("#id {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(tree.len(), 1, "Should have a single selector");

        match node[0] {
            SelectorElement::Name(name) => assert_eq!(name, "id"),
            _ => assert!(false, "Should have a name selector"),
        }

        assert!(rule.tokens.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_class_selector_no_property() {
        let rules = StyleSheetParser::parse(".class {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(tree.len(), 1, "Should have a single selector");

        match node[0] {
            SelectorElement::Class(name) => assert_eq!(name, "class"),
            _ => assert!(false, "Should have a class selector"),
        }

        assert!(rule.tokens.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_component_selector_no_property() {
        let rules = StyleSheetParser::parse("button {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(tree.len(), 1, "Should have a single selector");

        match node[0] {
            SelectorElement::Component(name) => assert_eq!(name, "button"),
            _ => assert!(false, "Should have a class selector"),
        }

        assert!(rule.tokens.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_complex_class_selector_no_property() {
        let rules = StyleSheetParser::parse(".a.b.c.d.e.f.g {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 7, "Should have a 7 selector class");

        use SelectorElement::*;
        let expected: SmallVec<[SelectorElement; 8]> = smallvec![
            Class("a".to_string()),
            Class("b".to_string()),
            Class("c".to_string()),
            Class("d".to_string()),
            Class("e".to_string()),
            Class("f".to_string()),
            Class("g".to_string()),
        ];

        expected
            .into_iter()
            .zip(node.into_iter())
            .for_each(|(expected, element)| {
                assert_eq!(expected, **element);
            });

        assert!(rule.tokens.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_single_composed_selector_no_property() {
        let rules = StyleSheetParser::parse("a.b#c.d {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 1, "Should have a single selector node");

        let node = &tree[0];
        assert_eq!(node.len(), 4, "Should have a 4 selectors");

        use SelectorElement::*;
        let expected: SmallVec<[SelectorElement; 8]> = smallvec![
            Component("a".to_string()),
            Class("b".to_string()),
            Name("c".to_string()),
            Class("d".to_string()),
        ];

        expected
            .into_iter()
            .zip(node.into_iter())
            .for_each(|(expected, element)| {
                assert_eq!(expected, **element);
            });

        assert!(rule.tokens.is_empty(), "Should have no token");
    }

    #[test]
    fn parse_multiple_composed_selector_no_property() {
        let rules = StyleSheetParser::parse("a.b #c .d e#f .g.h i j.k#l {}");
        assert_eq!(rules.len(), 1, "Should have a single rule");

        let rule = &rules[0];
        let tree = rule.selector.get_parent_tree();
        assert_eq!(tree.len(), 7, "Should have a single selector node");

        use SelectorElement::*;
        let expected: SmallVec<[SmallVec<[SelectorElement; 8]>; 8]> = smallvec![
            smallvec![Component("a".to_string()), Class("b".to_string())],
            smallvec![Name("c".to_string())],
            smallvec![Class("d".to_string())],
            smallvec![Component("e".to_string()), Name("f".to_string())],
            smallvec![Class("g".to_string()), Class("h".to_string())],
            smallvec![Component("i".to_string())],
            smallvec![
                Component("j".to_string()),
                Class("k".to_string()),
                Name("l".to_string())
            ],
        ];

        expected
            .into_iter()
            .zip(tree.into_iter())
            .for_each(|(node_expected, node)| {
                node_expected
                    .into_iter()
                    .zip(node)
                    .for_each(|(expected, element)| {
                        assert_eq!(expected, *element);
                    });
            });

        assert!(rule.tokens.is_empty(), "Should have no token");
    }
}
