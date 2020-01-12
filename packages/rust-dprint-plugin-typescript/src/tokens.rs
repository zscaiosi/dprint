use std::str;
use std::rc::Rc;
use super::*;
use swc_common::{BytePos, Span, SpanData};
use swc_ecma_parser::{token::{Token, TokenAndSpan, Word, Keyword, BinOpToken}};

pub struct TokenFinder {
    tokens: Rc<Vec<TokenAndSpan>>,
    file_bytes: Rc<Vec<u8>>,
    token_index: usize,
}

impl TokenFinder {
    pub fn new(tokens: Rc<Vec<TokenAndSpan>>, file_bytes: Rc<Vec<u8>>) -> TokenFinder {
        TokenFinder {
            tokens,
            file_bytes,
            token_index: 0,
        }
    }

    pub fn get_char_at(&self, pos: &BytePos) -> char {
        self.file_bytes[pos.0 as usize] as char
    }

    pub fn get_first_open_paren_token_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_within(node, |token| token.token == Token::LParen)
    }

    pub fn get_first_open_brace_token_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_within(node, |token| token.token == Token::LBrace)
    }

    pub fn get_first_open_bracket_token_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_within(node, |token| token.token == Token::LBracket)
    }

    pub fn get_first_comma_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_within(node, |token| token.token == Token::Comma)
    }

    pub fn get_first_semi_colon_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_within(node, |token| token.token == Token::Semi)
    }

    pub fn get_first_colon_token_after(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_after(node, |token| token.token == Token::Colon)
    }

    pub fn get_first_comma_after(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_after(node, |token| token.token == Token::Comma)
    }

    pub fn get_first_operator_after(&mut self, node: &dyn Ranged, operator_text: &str) -> Option<TokenAndSpan> {
        self.get_first_token_after_with_text(node, operator_text)
    }

    pub fn get_first_keyword_after(&mut self, node: &dyn Ranged, keyword_text: &str) -> Option<TokenAndSpan> {
        self.get_first_token_after_with_text(node, keyword_text)
    }

    fn get_first_token_after_with_text(&mut self, node: &dyn Ranged, text: &str) -> Option<TokenAndSpan> {
        let file_bytes = self.file_bytes.clone();
        self.get_first_token_after(node, |token| get_text(&file_bytes, &token.span.data()) == text)
    }

    pub fn get_first_else_keyword_within(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        let file_bytes = self.file_bytes.clone();
        self.get_first_token_within(node, |token| get_text(&file_bytes, &token.span.data()) == "else")
    }

    pub fn get_first_open_paren_token_before(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_before(node, |token| token.token == Token::LParen)
    }

    pub fn get_first_open_brace_token_before(&mut self, node: &dyn Ranged) -> Option<TokenAndSpan> {
        self.get_first_token_before(node, |token| token.token == Token::LBrace)
    }

    pub fn get_previous_token_end_before(&mut self, node: &dyn Ranged) -> BytePos {
        let previous_token = self.get_first_token_before(node, |item| true);
        if let Some(token) = previous_token {
            token.span.hi()
        } else {
            BytePos(0)
        }
    }

    pub fn get_next_token_pos_after(&mut self, node: &dyn Ranged) -> BytePos {
        let next_token = self.get_first_token_after(node, |item| true);
        if let Some(token) = next_token {
            token.span.lo()
        } else {
            BytePos(self.file_bytes.len() as u32)
        }
    }

    fn get_first_token_before<F>(&mut self, node: &dyn Ranged, is_match: F) -> Option<TokenAndSpan> where F : Fn(&TokenAndSpan) -> bool{
        let pos = node.lo();
        if self.tokens.is_empty() { return None; }
        self.move_to_node_pos(pos);

        if self.tokens[self.token_index].lo() < pos {
            let current_token = &self.tokens[self.token_index];
            if is_match(&current_token) {
                return Some(current_token.clone());
            }
        }

        while self.try_decrement_index() {
            let current_token = &self.tokens[self.token_index];
            if is_match(&current_token) {
                return Some(current_token.clone());
            }
        }

        return None;
    }

    fn get_first_token_after<F>(&mut self, node: &dyn Ranged, is_match: F) -> Option<TokenAndSpan> where F : Fn(&TokenAndSpan) -> bool {
        let end = node.hi();
        if self.tokens.is_empty() { return None; }
        self.move_to_node_end(end);

        while self.try_increment_index() {
            let current_token = &self.tokens[self.token_index];
            if is_match(&current_token) {
                return Some(current_token.clone());
            }
        }

        return None;
    }

    fn get_first_token_within<F>(&mut self, node: &dyn Ranged, is_match: F) -> Option<TokenAndSpan> where F : Fn(&TokenAndSpan) -> bool {
        let node_span_data = node.span().data();
        let pos = node_span_data.lo;
        let end = node_span_data.hi;
        if self.tokens.is_empty() { return None; }
        self.move_to_node_pos(pos);

        loop {
            let current_token = &self.tokens[self.token_index];
            let token_pos = current_token.span.data().lo;
            if token_pos >= end {
                break;
            } else if is_match(&current_token) {
                return Some(current_token.clone());
            }

            if !self.try_increment_index() {
                break;
            }
        }

        None
    }

    fn move_to_node_pos(&mut self, pos: BytePos) {
        while self.tokens[self.token_index].lo() < pos {
            if !self.try_increment_index() {
                break;
            }
        }

        while self.tokens[self.token_index].lo() > pos {
            if !self.try_decrement_index() {
                break;
            }
        }
    }

    fn move_to_node_end(&mut self, end: BytePos) {
        while self.tokens[self.token_index].hi() < end {
            if !self.try_increment_index() {
                break;
            }
        }

        while self.tokens[self.token_index].hi() > end {
            if !self.try_decrement_index() {
                break;
            }
        }
    }

    fn try_increment_index(&mut self) -> bool {
        if self.token_index == self.tokens.len() - 1 {
            false
        } else {
            self.token_index += 1;
            true
        }
    }

    fn try_decrement_index(&mut self) -> bool {
        if self.token_index == 0 {
            false
        } else {
            self.token_index -= 1;
            true
        }
    }

    fn get_text(&self, span_data: &SpanData) -> &str {
        get_text(&self.file_bytes, span_data)
    }
}

fn get_text<'a>(file_bytes: &'a Rc<Vec<u8>>, span_data: &SpanData) -> &'a str {
    let bytes = &file_bytes[(span_data.lo.0 as usize)..(span_data.hi.0 as usize)];
    str::from_utf8(&bytes).unwrap()
}
