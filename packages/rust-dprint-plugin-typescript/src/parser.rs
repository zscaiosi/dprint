extern crate dprint_core;

use dprint_core::*;
use std::collections::HashSet;
use super::*;
use swc_ecma_ast::{Module, ModuleItem, Stmt, Expr, Lit, Bool, JSXText, Number, Regex, Str};
use swc_common::{SpanData, comments::{Comment, CommentKind}};

pub fn parse(source_file: ParsedSourceFile) -> Vec<PrintItem> {
    let mut context = Context {
        config: TypeScriptConfiguration {
            single_quotes: false
        },
        comments: source_file.comments,
        file_bytes: source_file.file_bytes,
        current_node: Node::Module(source_file.module.clone()),
        parent_stack: Vec::new(),
        handled_comments: HashSet::new(),
    };
    parse_node(Node::Module(source_file.module), &mut context)
}

fn parse_module_item(item: ModuleItem, context: &mut Context) -> Vec<PrintItem> {
    match item {
        ModuleItem::Stmt(node) => parse_stmt(node, context),
        _ => Vec::new(), // todo: remove this
    }
}

fn parse_stmt(stmt: Stmt, context: &mut Context) -> Vec<PrintItem> {
    match stmt {
        Stmt::Expr(node) => parse_expr(*node, context),
        _ => Vec::new(), // todo: remove this
    }
}

fn parse_expr(expr: Expr, context: &mut Context) -> Vec<PrintItem> {
    match expr {
        Expr::Lit(lit) => parse_literal(lit, context),
        _ => Vec::new(), // todo: remove this
    }
}

fn parse_literal(lit: Lit, context: &mut Context) -> Vec<PrintItem> {
    match lit {
        Lit::Bool(node) => parse_node(Node::Bool(node), context),
        Lit::JSXText(node) => parse_node(Node::JsxText(node), context),
        Lit::Null(node) => parse_node(Node::Null(node), context),
        Lit::Num(node) => parse_node(Node::Num(node), context),
        Lit::Regex(node) => parse_node(Node::Regex(node), context),
        Lit::Str(node) => parse_node(Node::Str(node), context),
    }
}

fn parse_node(node: Node, context: &mut Context) -> Vec<PrintItem> {
    parse_node_with_inner_parse(node, context, |items| items)
}

fn parse_node_with_inner_parse(node: Node, context: &mut Context, inner_parse: impl Fn(Vec<PrintItem>) -> Vec<PrintItem> + Clone + 'static) -> Vec<PrintItem> {
    // store info
    let past_current_node = std::mem::replace(&mut context.current_node, node.clone());
    context.parent_stack.push(past_current_node);

    // comments

    // now parse items
    let items = inner_parse(parse_node(node, context));

    // pop info
    context.current_node = context.parent_stack.pop().unwrap();

    return items;

    fn parse_node(node: Node, context: &mut Context) -> Vec<PrintItem> {
        match node {
            Node::Bool(node) => parse_bool_literal(&node),
            Node::JsxText(node) => parse_jsx_text(&node, context),
            Node::Null(_) => vec!["null".into()],
            Node::Num(node) => parse_num_literal(&node, context),
            Node::Regex(node) => parse_reg_exp_literal(&node, context),
            Node::Str(node) => parse_string_literal(&node, context),
            Node::Module(node) => parse_module(node, context),
        }
    }
}

/* Module */

fn parse_module(node: Module, context: &mut Context) -> Vec<PrintItem> {
    let mut items = Vec::new();
    for item in node.body {
        items.extend(parse_module_item(item, context));
    }
    items
}

/* Literals */

fn parse_bool_literal(node: &Bool) -> Vec<PrintItem> {
    vec![match node.value {
        true => "true",
        false => "false",
    }.into()]
}

fn parse_jsx_text(node: &JSXText, context: &mut Context) -> Vec<PrintItem> {
    vec![]
}

fn parse_num_literal(node: &Number, context: &mut Context) -> Vec<PrintItem> {
    vec![context.get_span_text(&node.span.data()).into()]
}

fn parse_reg_exp_literal(node: &Regex, context: &mut Context) -> Vec<PrintItem> {
    // the exp and flags should not be nodes so just ignore that (swc issue #511)
    let mut items = Vec::new();
    items.push("/".into());
    items.push(context.get_span_text(&node.exp.span.data()).into());
    items.push("/".into());
    if let Some(flags) = &node.flags {
        items.push(context.get_span_text(&flags.span.data()).into());
    }
    items
}

fn parse_string_literal(node: &Str, context: &mut Context) -> Vec<PrintItem> {
    return parser_helpers::parse_raw_string(&get_string_literal_text(&node.span.data(), context));

    fn get_string_literal_text(span_data: &SpanData, context: &mut Context) -> String {
        let string_value = get_string_value(&span_data, context);

        return match context.config.single_quotes {
            true => format!("'{}'", string_value.replace("'", "\\'")),
            false => format!("\"{}\"", string_value.replace("\"", "\\\"")),
        };

        fn get_string_value(span_data: &SpanData, context: &mut Context) -> String {
            let raw_string_text = context.get_span_text(&span_data);
            let string_value = raw_string_text.chars().skip(1).take(raw_string_text.chars().count() - 2).collect::<String>();
            let is_double_quote = string_value.chars().next().unwrap() == '"';

            match is_double_quote {
                true => string_value.replace("\\\"", "\""),
                false => string_value.replace("\\'", "'"),
            }
        }
    }
}

/* Comments */

fn parse_leading_comments(span_data: SpanData, context: &mut Context) -> Vec<PrintItem> {
    let leading_comments = context.comments.leading_comments(span_data.lo).map(|c| c.clone());
    parse_comments_as_leading(leading_comments, context)
}

fn parse_comments_as_leading(optional_comments: Option<Vec<Comment>>, context: &mut Context) -> Vec<PrintItem> {
    if optional_comments.is_none() {
        return vec![];
    }

    let comments = optional_comments.unwrap();
    if comments.is_empty() {
        return vec![];
    }

    let last_comment = comments.last().unwrap();
    let last_previously_handled = context.has_handled_comment(&last_comment.span.data());
    let items = Vec::new();

    items.extend(parse_comment_collection(&comments, Option::None, context));

    if (!last_previously_handled) {
        // todo
    }

    items
}

fn parse_comment_collection(comments: &Vec<Comment>, last_span_data: Option<SpanData>, context: &mut Context) -> Vec<PrintItem> {

}

fn parse_comment(comment: &Comment, context: &mut Context) -> Vec<PrintItem> {
    // only parse if handled
    let comment_span_data = comment.span.data();
    if context.has_handled_comment(&comment_span_data) {
        return Vec::new();
    }

    // mark handled and parse
    context.mark_comment_handled(&comment_span_data);
    return match comment.kind {
        CommentKind::Block => parse_comment_block(comment),
        CommentKind::Line => parse_comment_line(comment),
    };

    fn parse_comment_block(comment: &Comment) -> Vec<PrintItem> {
        let mut vec = Vec::new();
        vec.push("/*".into());
        vec.extend(parser_helpers::parse_raw_string(&comment.text));
        vec.push("*/".into());
        vec
    }

    fn parse_comment_line(comment: &Comment) -> Vec<PrintItem> {
        return vec![
            get_comment_text(&comment.text).into(),
            PrintItem::ExpectNewLine
        ];

        fn get_comment_text(original_text: &String) -> String {
            let non_slash_index = get_first_non_slash_index(&original_text);
            let start_text_index = if original_text.chars().skip(non_slash_index).next() == Some(' ') { non_slash_index + 1 } else { non_slash_index };
            let comment_text_original = original_text.chars().skip(start_text_index).collect::<String>();
            let comment_text = comment_text_original.trim_end();
            let prefix = format!("//{}", original_text.chars().take(non_slash_index).collect::<String>());

            return if comment_text.is_empty() {
                prefix
            } else {
                format!("{} {}", prefix, comment_text)
            };

            fn get_first_non_slash_index(text: &String) -> usize {
                let mut i: usize = 0;
                for c in text.chars() {
                    if c != '/' {
                        return i;
                    }
                    i += 1;
                }

                return i;
            }
        }
    }
}