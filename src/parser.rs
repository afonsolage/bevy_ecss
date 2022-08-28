use std::{error::Error, fmt::Display};

use bevy::prelude::error;
use cssparser::{
    AtRuleParser, DeclarationListParser, DeclarationParser, ParseError, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, ToCss, Token,
};
use smallvec::SmallVec;

use crate::{
    property::Property,
    selector::{Selector, SelectorElement},
    stylesheet::StyleRule,
};

#[derive(Debug)]
pub enum EcssError {
    UnsupportedSelector,
    UnsupportedProperty(String),
    InvalidPropertyValue(String),
}

impl Error for EcssError {}

impl Display for EcssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcssError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            EcssError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            EcssError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
        }
    }
}

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
        let mut properties = SmallVec::new();

        for property in DeclarationListParser::new(input, PropertyParser) {
            match property {
                Ok(property) => properties.push(property),
                Err((err, a)) => println!("Failed: {:?} ({})", err, a),
            }
        }

        Ok(StyleRule(prelude, properties))
    }
}

fn parse_selector<'i, 'tt>(
    parser: &mut Parser<'i, 'tt>,
) -> Result<Selector, ParseError<'i, EcssError>> {
    let mut values = vec![];

    let mut next_is_class = false;

    while let Ok(token) = parser.next_including_whitespace() {
        use cssparser::Token::*;
        match token {
            Ident(v) => {
                if next_is_class {
                    next_is_class = false;
                    values.push(SelectorElement::Class(v.to_string()));
                } else {
                    values.push(SelectorElement::Component(v.to_string()));
                }
            }
            IDHash(v) => values.push(SelectorElement::Name(v.to_string())),
            WhiteSpace(_) => values.push(SelectorElement::Child),
            Delim(c) if *c == '.' => next_is_class = true,
            _ => {
                println!("Unexpected token: {:?}", token);
            }
        }
    }

    // Remove noise the trailing white spaces, if any
    while values.len() > 0 && values.last().unwrap() == &SelectorElement::Child {
        values.remove(values.len() - 1);
    }

    Ok(Selector(values))
}

impl<'i> AtRuleParser<'i> for StyleSheetParser {
    type Prelude = ();

    type AtRule = StyleRule;

    type Error = EcssError;
}

struct PropertyParser;

impl<'i> DeclarationParser<'i> for PropertyParser {
    type Declaration = Property;

    type Error = EcssError;

    fn parse_value<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        parser: &mut Parser<'i, 't>,
    ) -> Result<Self::Declaration, ParseError<'i, EcssError>> {
        match Property::new(&name, parse_values(parser)?.into()) {
            Ok(p) => Ok(p),
            Err(err) => Err(parser.new_custom_error(err)),
        }
    }
}

impl<'i> AtRuleParser<'i> for PropertyParser {
    type Prelude = ();
    type AtRule = Property;
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
