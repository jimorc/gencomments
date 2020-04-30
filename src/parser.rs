use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum LexItem {
    WhiteSpace(String),
    SingleComment(String),
    OuterLineDocComment(String),
    MultilineComment(String),
}

pub struct Parser {}

impl Parser {
    pub fn lex(&self, code: &str) -> Result<Vec<LexItem>, String>{
        let mut result = Vec::new();
        let mut it = code.chars().peekable();
        while let Some(&c) = it.peek() {
            match c {
                // There seems to be no consensus as to what Rust should allow
                // as acceptable white space characters separating tokens.
                // So fo now, Parser accepts all Pattern_White_Space.
                ' ' | '\n' | '\r' | '\t' | '\x0B' | '\x0C' | '\u{85}' | 
                '\u{200E}' | '\u{200F}' | '\u{2028}' | '\u{2029}'=> {
                    result.push(self.lex_whitespace(&mut it));
                }
                '/' => {
                    result.push(self.lex_forward_slash(&mut it));
                }
                _ => {
                    return Err(format!("Character: '{}' cannot be lexed", c));
                }
            }
        }
        Ok(result)
    }

    fn lex_whitespace<T: Iterator<Item = char>>(&self, iterator: &mut Peekable<T>) 
        -> LexItem {
        let mut whitespace = String::new();
        while let Some(&c) = iterator.peek() {
            match c {
            // There seems to be no consensus as to what Rust should allow
            // as acceptable white space characters separating tokens.
            // So fo now, Parser accepts all Pattern_White_Space.
                ' ' | '\n' | '\r' | '\t' | '\x0B' | '\x0C' | '\u{85}' | 
                    '\u{200E}' | '\u{200F}' | '\u{2028}' | '\u{2029}'=> {
                    whitespace.push(c);
                    iterator.next();
                }
                _ => {  // we have reached the end of the whitespace
                    break;
                }
            }
        }
        LexItem::WhiteSpace(whitespace) 
    }

    // this should be called only if the first character pointed to by the iterator
    // is a '/'.
    fn lex_forward_slash<T: Iterator<Item = char>>(&self, iterator: &mut Peekable<T>) 
        -> LexItem {
        let mut comment = String::new();

        let &c = iterator.peek().unwrap();
        if c == '/' {
            comment.push(c);
            iterator.next();
            let &c2 = iterator.peek().unwrap();
            match c2 {
                // handle comment that starts with at least '//'
                '/' => {
                    comment.push(c2);
                    iterator.next();
                    let &c3 = iterator.peek().unwrap();
                    match c3 {
                        '/' => {
                            // handle comment that starts with at least '///'
                            self.lex_at_least_3_slashes(&mut comment, &mut *iterator)
                         }
                        _ => {
                            self.get_comment_text(&mut comment, &mut *iterator);
                            LexItem::SingleComment(comment) 
                        }
                    }
                }
                '*' => {
                    comment.push(c2);
                    iterator.next();
                    self.get_multiline_comment_text(&mut comment, &mut *iterator);
                    LexItem::MultilineComment(comment)
                }
                _ => {
                    panic!("Suspected comment does not begin with '//' or '/*'.");
                }
            }
        } else {
            panic!("lex_forward_slash was called but first character was not '/'.")
        }
    }

    pub fn get_comment_text<T: Iterator<Item = char>>(&self, comment: &mut String, iterator: &mut Peekable<T>) {
        while let Some(c) = iterator.next() {
            comment.push(c);
            if c == '\n' {
                break;
            }
        }

    }

    fn get_multiline_comment_text<T: Iterator<Item = char>>(&self, comment: &mut String, iterator: &mut Peekable<T>) {
        while let Some(c) = iterator.next() {
            comment.push(c);
            if c == '*' {
                let &c2 = iterator.peek().unwrap();
                if c2 == '/' {
                    comment.push(c2);
                    iterator.next();
                    break;
                }
            }
        }
    }

    // handle case where comment begins with at least '///'
    fn lex_at_least_3_slashes<T: Iterator<Item = char>>(&self, comment: &mut String, iterator: &mut Peekable<T>) 
        -> LexItem {
        let &c3 = iterator.peek().unwrap();
        comment.push(c3);
        iterator.next();
        if let Some(c4) = iterator.next() {
            comment.push(c4);
            self.get_comment_text(comment, iterator);
            if c4 == '/' {
                // handle ////
                LexItem::SingleComment(comment.clone())   
            } else {
                // handle ///
                LexItem::OuterLineDocComment(comment.clone()) 
            }
        } else {
            panic!("Programming error: should never reach here.");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn test_lex_whitespace() {
        let parser = Parser {};
        let mut whitespace = String::from(" \n\r\t");
        whitespace.push('\x0B');
        whitespace.push('\x0C');
        whitespace.push('\u{85}');
        whitespace.push('\u{200E}');
        whitespace.push('\u{200F}');
        whitespace.push('\u{2028}');
        whitespace.push('\u{2029}');
        let mut iter = whitespace.chars().peekable();
        if let LexItem::WhiteSpace(spaces) = parser.lex_whitespace(&mut iter) {
            assert_eq!(spaces, whitespace.to_string());
        } else {
            panic!("Call to lex_whitespace did not return a WhiteSpace.");
        }
    }

    #[test]
    fn test_lex() {
        let parser = Parser {};
        let code = String::from(" \n\r\t// A comment\n");
        let items = parser.lex(&code).unwrap();
        assert_eq!(items.len(), 2);
        if let LexItem::WhiteSpace(wspace) = items.get(0).unwrap() {
            assert_eq!(wspace.as_str(), " \n\r\t");
        } else {
            panic!("Call to lex did not return a WhiteSpace.");
        }
        if let LexItem::SingleComment(comment) = items.get(1).unwrap() {
            assert_eq!(comment.as_str(), "// A comment\n");
        } else {
            panic!("Call to lex did not return a SingleComment.");
        }
    }

    #[test]
    fn test_lex_single_comment_space() {
        let parser = Parser {};
        let comment = String::from("// A comment  \n");
        let comm = comment.clone() + " ";
        let mut it = comm.chars().peekable();
        if let LexItem::SingleComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_forward_slash did not return a SingleComment.");
        }
    }

    #[test]
    fn test_lex_single_comment_tab() {
        let parser = Parser {};
        let comment = String::from("//\tA comment  \n");
        let comm = comment.clone() + " ";
        let mut it = comm.chars().peekable();
        if let LexItem::SingleComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_forward_slash did not return a SingleComment.");
        }
    }

    #[test]
    fn test_lex_single_comment_new_line() {
        let parser = Parser {};
        let comment = String::from("//\n");
        let comm = comment.clone() + " ";
        let mut it = comm.chars().peekable();
        if let LexItem::SingleComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_forward_slash did not return a SingleComment.");
        }
    }

    #[test]
    fn test_lex_single_comment_no_whitespace() {
        let parser = Parser {};
        let comment = String::from("//a");
        let mut it = comment.chars().peekable();
        if let LexItem::SingleComment(comment2) = parser.lex_forward_slash(&mut it) {
            assert_eq!(comment, comment2);
         } else {
             panic!("Call to lex_forward_slash did not return a SingleComment.");
         }
     }

    #[test]
    #[should_panic(expected = "Suspected comment does not begin with \'//\' or \'/*\'.")]
    fn test_lex_forward_slash_invalid_input() {
        let parser = Parser {};
        let comment = String::from("/a");
        let mut it = comment.chars().peekable();
        let _comment2 = parser.lex_forward_slash(&mut it); 
    }

    #[test]
    #[should_panic(expected = "lex_forward_slash was called but first character was not \'/\'.")]
    fn test_lex_forward_slash_no_slash() {
        let parser = Parser {};
        let comment = String::from("a");
        let mut it = comment.chars().peekable();
        let _comment2 = parser.lex_forward_slash(&mut it);
        panic!("test_lex_forward_slash_no_slash did not panic.");
    }

    #[test]
    fn test_lex_outer_linedoc_comment() {
        let parser = Parser {};
        let comment = String::from("/// A Doc comment  \n");
        let mut it = comment.chars().peekable();
        if let LexItem::OuterLineDocComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_forward_slash did not return a OuterLineDocComment.");
        }
    }

   #[test]
    fn test_multiline_comment() {
        let parser = Parser {};
        let comment = String::from("/* A Doc comment  \n * Second line*/");
        let mut it = comment.chars().peekable();
        if let LexItem::MultilineComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_forward_slash did not return a MultilineComment.");
        }
    }

    #[test]
    fn test_4_slashes() {
        let parser = Parser {};
        let comment = String::from("////\n");
        let mut it = comment.chars().peekable();
        if let LexItem::SingleComment(comment2) = parser.lex_forward_slash(&mut it) {
           assert_eq!(comment, comment2);
        } else {
            panic!("Call to lex_4_slashes did not return a SingleComment.");
        }
    }
}
