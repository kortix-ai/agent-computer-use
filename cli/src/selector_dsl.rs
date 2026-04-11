use agent_computer_use_core::node::Role;
use agent_computer_use_core::selector::{Selector, SelectorChain};

pub fn parse(input: &str) -> Result<SelectorChain, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("empty selector".to_string());
    }

    let segments: Vec<&str> = split_chain(input);
    let mut selectors = Vec::with_capacity(segments.len());

    for segment in segments {
        let segment = segment.trim();
        if segment.is_empty() {
            return Err("empty selector segment (check >> usage)".to_string());
        }
        selectors.push(parse_segment(segment)?);
    }

    Ok(SelectorChain { selectors })
}

fn split_chain(input: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + 4 <= len && &input[i..i + 4] == " >> " {
            segments.push(&input[start..i]);
            start = i + 4;
            i = start;
        } else {
            i += 1;
        }
    }
    segments.push(&input[start..]);
    segments
}

fn parse_segment(input: &str) -> Result<Selector, String> {
    let tokens = tokenize(input)?;
    let mut selector = Selector::new();

    for token in tokens {
        match token {
            Token::KeyValue(key, value) => match key {
                "role" => {
                    selector.role = Some(Role::parse(&value));
                }
                "name" => {
                    selector.name = Some(value);
                }
                "name~" => {
                    selector.name_contains = Some(value);
                }
                "id" => {
                    selector.id = Some(value);
                }
                "id~" => {
                    selector.id_contains = Some(value);
                }
                "app" => {
                    selector.app = Some(value);
                }
                "depth" => {
                    selector.max_depth = Some(
                        value
                            .parse::<u32>()
                            .map_err(|_| format!("invalid depth value: '{value}'"))?,
                    );
                }
                "index" => {
                    selector.index = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| format!("invalid index value: '{value}'"))?,
                    );
                }
                "css" => {
                    selector.css = Some(value);
                }
                "data-testid" => {
                    selector.id = Some(value);
                }
                _ => {
                    return Err(format!("unknown filter key: '{key}'"));
                }
            },
            Token::BareWord(word) => {
                selector.role = Some(Role::parse(&word));
            }
            Token::QuotedString(s) => {
                selector.name = Some(s);
            }
        }
    }

    Ok(selector)
}

#[derive(Debug)]
enum Token<'a> {
    KeyValue(&'a str, String),
    BareWord(String),
    QuotedString(String),
}

fn tokenize(input: &str) -> Result<Vec<Token<'_>>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(i, ch)) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        if ch == '"' {
            chars.next();
            let s = consume_until_quote(&mut chars)?;
            tokens.push(Token::QuotedString(s));
            continue;
        }

        let word_start = i;
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() || c == '"' {
                break;
            }
            if c == '=' {
                break;
            }
            chars.next();
        }

        let word_end = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());
        let word = &input[word_start..word_end];

        if let Some(&(_, '=')) = chars.peek() {
            chars.next();
            let key = word;
            let value = if let Some(&(_, '"')) = chars.peek() {
                chars.next();
                consume_until_quote(&mut chars)?
            } else {
                let val_start = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    chars.next();
                }
                let val_end = chars.peek().map(|&(i, _)| i).unwrap_or(input.len());
                input[val_start..val_end].to_string()
            };

            tokens.push(Token::KeyValue(key, value));
        } else {
            tokens.push(Token::BareWord(word.to_string()));
        }
    }

    Ok(tokens)
}

fn consume_until_quote(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
) -> Result<String, String> {
    let mut s = String::new();
    loop {
        match chars.next() {
            Some((_, '"')) => return Ok(s),
            Some((_, '\\')) => match chars.next() {
                Some((_, c)) => s.push(c),
                None => return Err("unexpected end of string after backslash".to_string()),
            },
            Some((_, c)) => s.push(c),
            None => return Err("unterminated quoted string".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_computer_use_core::node::Role;

    #[test]
    fn test_simple_role() {
        let chain = parse("role=button").unwrap();
        assert!(chain.is_simple());
        assert_eq!(chain.first().role, Some(Role::Button));
    }

    #[test]
    fn test_role_and_name() {
        let chain = parse(r#"role=button name="Login""#).unwrap();
        assert!(chain.is_simple());
        assert_eq!(chain.first().role, Some(Role::Button));
        assert_eq!(chain.first().name, Some("Login".to_string()));
    }

    #[test]
    fn test_shorthand_role() {
        let chain = parse("button").unwrap();
        assert_eq!(chain.first().role, Some(Role::Button));
    }

    #[test]
    fn test_shorthand_name() {
        let chain = parse(r#""Login""#).unwrap();
        assert_eq!(chain.first().name, Some("Login".to_string()));
    }

    #[test]
    fn test_shorthand_both() {
        let chain = parse(r#"button "Submit""#).unwrap();
        assert_eq!(chain.first().role, Some(Role::Button));
        assert_eq!(chain.first().name, Some("Submit".to_string()));
    }

    #[test]
    fn test_name_contains() {
        let chain = parse(r#"name~="email""#).unwrap();
        assert_eq!(chain.first().name_contains, Some("email".to_string()));
    }

    #[test]
    fn test_chain() {
        let chain = parse(r#"role=form >> role=button name="Submit""#).unwrap();
        assert_eq!(chain.selectors.len(), 2);
        assert_eq!(chain.selectors[0].role, Some(Role::Form));
        assert_eq!(chain.selectors[1].role, Some(Role::Button));
        assert_eq!(chain.selectors[1].name, Some("Submit".to_string()));
    }

    #[test]
    fn test_app_scoped() {
        let chain = parse(r#"app="Firefox" role=button name="Submit""#).unwrap();
        assert_eq!(chain.first().app, Some("Firefox".to_string()));
        assert_eq!(chain.first().role, Some(Role::Button));
    }

    #[test]
    fn test_id() {
        let chain = parse(r#"id="login-btn""#).unwrap();
        assert_eq!(chain.first().id, Some("login-btn".to_string()));
    }

    #[test]
    fn test_depth() {
        let chain = parse("role=button depth=3").unwrap();
        assert_eq!(chain.first().max_depth, Some(3));
    }

    #[test]
    fn test_unquoted_values() {
        let chain = parse("role=button name=Submit").unwrap();
        assert_eq!(chain.first().role, Some(Role::Button));
        assert_eq!(chain.first().name, Some("Submit".to_string()));
    }

    #[test]
    fn test_empty_error() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_empty_chain_segment() {
        assert!(parse("role=button >>  >> role=form").is_err());
    }

    #[test]
    fn test_escaped_quote() {
        let chain = parse(r#"name="say \"hello\"""#).unwrap();
        assert_eq!(chain.first().name, Some(r#"say "hello""#.to_string()));
    }

    #[test]
    fn test_index() {
        let chain = parse("role=button index=2").unwrap();
        assert_eq!(chain.first().role, Some(Role::Button));
        assert_eq!(chain.first().index, Some(2));
    }

    #[test]
    fn test_index_in_chain() {
        let chain = parse(r#"name="List" >> role=button index=0"#).unwrap();
        assert_eq!(chain.selectors.len(), 2);
        assert_eq!(chain.selectors[1].role, Some(Role::Button));
        assert_eq!(chain.selectors[1].index, Some(0));
    }
}
