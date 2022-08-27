use std::{error::Error, fmt::Display};

use cssparser::{
    AtRuleParser, DeclarationListParser, DeclarationParser, ParseError, Parser, ParserInput,
    QualifiedRuleParser, RuleListParser, Token,
};

use crate::element::{Property, Selector, SelectorElement};

pub fn parse_stylesheets(content: &str) -> Vec<StyleRule> {
    let mut rules = vec![];

    let mut input = ParserInput::new(content);
    let mut parser = Parser::new(&mut input);

    let a = RuleListParser::new_for_stylesheet(&mut parser, StylesheetParser);
    for rule in a {
        match rule {
            Ok(rule) => rules.push(rule),
            Err((err, a)) => println!("Error parsing rule: {:?} ({})", err, a),
        }
    }

    rules
}

#[derive(Debug, Clone)]
pub struct StyleRule(pub Selector, pub Vec<Property>);

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

struct StylesheetParser;

impl<'i> QualifiedRuleParser<'i> for StylesheetParser {
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
        let mut properties = vec![];

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

impl<'i> AtRuleParser<'i> for StylesheetParser {
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
) -> Result<Vec<Token<'i>>, ParseError<'i, EcssError>> {
    let mut values = vec![];

    while let Ok(token) = parser.next_including_whitespace() {
        values.push(token.clone())
    }

    Ok(values)
}
