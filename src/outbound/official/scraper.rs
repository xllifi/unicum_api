use std::str::Chars;

use log::trace;

use crate::{entities, outbound::official::ScraperError};

pub fn parse_required_attr<'a>(
    element: scraper::ElementRef<'a>,
    attribute: &str,
) -> Result<&'a str, ScraperError> {
    element
        .value()
        .attr(attribute)
        .ok_or(ScraperError::MissingAttribute {
            attribute: attribute.to_owned(),
        })
}

pub fn parse_u8_attr<'a>(
    element: scraper::ElementRef<'a>,
    attribute: &str,
) -> Result<u8, ScraperError> {
    parse_required_attr(element, attribute)?
        .parse::<u8>()
        .map_err(|error| ScraperError::InvalidValue {
            field: attribute.to_owned(),
            value: error.to_string(),
        })
}
pub fn to_digit_next(chars: &mut Chars, radix: u32) -> Result<u32, ScraperError> {
    let next = chars.next();
    trace!("Trying to convert {next:?}");

    let character = next.ok_or(ScraperError::MissingElement {
        element: "character".into(),
    })?;
    character
        .to_digit(radix)
        .ok_or_else(|| ScraperError::InvalidValue {
            field: "character".into(),
            value: format!("{character:?}"),
        })
}

impl From<time::Date> for entities::Date {
    fn from(value: time::Date) -> Self {
        Self {
            day: value.day(),
            month: value.month() as u8,
            year: value.year(),
        }
    }
}

pub fn sel_to_row_and_col<S: AsRef<str>>(sel: S) -> Result<(u8, u8), ScraperError> {
    let mut chars = sel.as_ref().chars();
    let row: u8 = to_digit_next(&mut chars, 16)? as u8;
    let col: u8 = to_digit_next(&mut chars, 16)? as u8;

    Ok((row, col))
}

#[cfg(test)]
mod tests {
    mod parse_required_attr {
        use scraper::{Html, Selector};

        use crate::outbound::official::scraper::parse_required_attr;

        #[test]
        fn path_ok_attribute_exists() {
            // Given
            let document = Html::parse_fragment(r#"<input name="selection" value="A1">"#);
            let selector = Selector::parse("input").expect("selector should be valid");
            let element = document
                .select(&selector)
                .next()
                .expect("input should exist");

            // When
            let result = parse_required_attr(element, "value");

            // Then
            assert!(result.is_ok());
            assert_eq!("A1", result.expect("attribute should be returned"));
        }

        #[test]
        fn path_err_attribute_missing() {
            // Given
            let document = Html::parse_fragment(r#"<input name="selection">"#);
            let selector = Selector::parse("input").expect("selector should be valid");
            let element = document
                .select(&selector)
                .next()
                .expect("input should exist");

            // When
            let result = parse_required_attr(element, "value");

            // Then
            assert!(result.is_err());
            match result.expect_err("missing attribute should fail") {
                crate::outbound::official::ScraperError::MissingAttribute { attribute } => {
                    assert_eq!("value", attribute);
                }
                error => panic!("expected MissingAttribute, got {error:?}"),
            }
        }
    }

    mod parse_u8_attr {
        use scraper::{Html, Selector};

        use crate::outbound::official::{ScraperError, scraper::parse_u8_attr};

        fn input_element(document: &Html) -> scraper::ElementRef<'_> {
            let selector = Selector::parse("input").expect("selector should be valid");
            document
                .select(&selector)
                .next()
                .expect("input should exist")
        }

        #[test]
        fn path_ok_attribute_is_u8() {
            // Given
            let document = Html::parse_fragment(r#"<input value="255">"#);
            let element = input_element(&document);

            // When
            let result = parse_u8_attr(element, "value");

            // Then
            assert!(result.is_ok());
            assert_eq!(255, result.expect("attribute should parse as u8"));
        }

        #[test]
        fn path_err_attribute_missing() {
            // Given
            let document = Html::parse_fragment("<input>");
            let element = input_element(&document);

            // When
            let result = parse_u8_attr(element, "value");

            // Then
            assert!(result.is_err());
            match result.expect_err("missing attribute should fail") {
                ScraperError::MissingAttribute { attribute } => {
                    assert_eq!("value", attribute);
                }
                error => panic!("expected MissingAttribute, got {error:?}"),
            }
        }

        #[test]
        fn path_err_attribute_is_not_a_number() {
            // Given
            let document = Html::parse_fragment(r#"<input value="many">"#);
            let element = input_element(&document);

            // When
            let result = parse_u8_attr(element, "value");

            // Then
            assert!(result.is_err());
            match result.expect_err("non-numeric attribute should fail") {
                ScraperError::InvalidValue { field, value } => {
                    assert_eq!("value", field);
                    assert_eq!("invalid digit found in string", value);
                }
                error => panic!("expected InvalidValue, got {error:?}"),
            }
        }

        #[test]
        fn path_err_attribute_exceeds_u8() {
            // Given
            let document = Html::parse_fragment(r#"<input value="256">"#);
            let element = input_element(&document);

            // When
            let result = parse_u8_attr(element, "value");

            // Then
            assert!(result.is_err());
            match result.expect_err("out-of-range attribute should fail") {
                ScraperError::InvalidValue { field, value } => {
                    assert_eq!("value", field);
                    assert_eq!("number too large to fit in target type", value);
                }
                error => panic!("expected InvalidValue, got {error:?}"),
            }
        }
    }

    mod to_digit_next {
        use crate::outbound::official::{ScraperError, scraper::to_digit_next};

        #[test]
        fn path_ok_decimal_character() {
            // Given
            let mut characters = "9x".chars();

            // When
            let result = to_digit_next(&mut characters, 10);

            // Then
            assert!(result.is_ok());
            assert_eq!(9, result.expect("decimal character should parse"));
            assert_eq!(Some('x'), characters.next());
        }

        #[test]
        fn path_ok_hexadecimal_character() {
            // Given
            let mut characters = "F".chars();

            // When
            let result = to_digit_next(&mut characters, 16);

            // Then
            assert!(result.is_ok());
            assert_eq!(15, result.expect("hexadecimal character should parse"));
            assert_eq!(None, characters.next());
        }

        #[test]
        fn path_err_iterator_is_empty() {
            // Given
            let mut characters = "".chars();

            // When
            let result = to_digit_next(&mut characters, 16);

            // Then
            assert!(result.is_err());
            match result.expect_err("empty iterator should fail") {
                ScraperError::MissingElement { element } => {
                    assert_eq!("character", element);
                }
                error => panic!("expected MissingElement, got {error:?}"),
            }
        }

        #[test]
        fn path_err_character_is_invalid_for_radix() {
            // Given
            let mut characters = "A".chars();

            // When
            let result = to_digit_next(&mut characters, 10);

            // Then
            assert!(result.is_err());
            match result.expect_err("non-decimal character should fail") {
                ScraperError::InvalidValue { field, value } => {
                    assert_eq!("character", field);
                    assert_eq!("'A'", value);
                }
                error => panic!("expected InvalidValue, got {error:?}"),
            }
        }
    }

    mod sel_to_row_and_col {
        use crate::outbound::official::{ScraperError, scraper::sel_to_row_and_col};

        #[test]
        fn path_ok_hexadecimal_selection() {
            // Given
            let selection = "AF";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_ok());
            assert_eq!((10, 15), result.expect("selection should parse"));
        }

        #[test]
        fn anxiety_trailing_characters_are_ignored() {
            // Given
            let selection = "12extra";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_ok());
            assert_eq!((1, 2), result.expect("first two characters should parse"));
        }

        #[test]
        fn path_err_selection_is_empty() {
            // Given
            let selection = "";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_err());
            match result.expect_err("empty selection should fail") {
                ScraperError::MissingElement { element } => {
                    assert_eq!("character", element);
                }
                error => panic!("expected MissingElement, got {error:?}"),
            }
        }

        #[test]
        fn path_err_selection_has_only_row() {
            // Given
            let selection = "A";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_err());
            match result.expect_err("selection without column should fail") {
                ScraperError::MissingElement { element } => {
                    assert_eq!("character", element);
                }
                error => panic!("expected MissingElement, got {error:?}"),
            }
        }

        #[test]
        fn path_err_row_is_not_hexadecimal() {
            // Given
            let selection = "G1";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_err());
            match result.expect_err("invalid row should fail") {
                ScraperError::InvalidValue { field, value } => {
                    assert_eq!("character", field);
                    assert_eq!("'G'", value);
                }
                error => panic!("expected InvalidValue, got {error:?}"),
            }
        }

        #[test]
        fn path_err_column_is_not_hexadecimal() {
            // Given
            let selection = "1G";

            // When
            let result = sel_to_row_and_col(selection);

            // Then
            assert!(result.is_err());
            match result.expect_err("invalid column should fail") {
                ScraperError::InvalidValue { field, value } => {
                    assert_eq!("character", field);
                    assert_eq!("'G'", value);
                }
                error => panic!("expected InvalidValue, got {error:?}"),
            }
        }
    }
}
