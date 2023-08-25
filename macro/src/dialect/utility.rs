use super::error::Error;
use comrak::{arena_tree::NodeEdge, format_commonmark, nodes::NodeValue, parse_document, Arena};
use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;

const RESERVED_NAMES: &[&str] = &["name", "operation", "builder"];

pub fn sanitize_snake_case_name(name: &str) -> Ident {
    sanitize_name(&name.to_case(Case::Snake))
}

fn sanitize_name(name: &str) -> Ident {
    // Replace any "." with "_"
    let mut name = name.replace('.', "_");

    // Add "_" suffix to avoid conflicts with existing methods
    if RESERVED_NAMES.contains(&name.as_str())
        || name
            .chars()
            .next()
            .expect("name has at least one char")
            .is_numeric()
    {
        name = format!("_{}", name);
    }

    // Try to parse the string as an ident, and prefix the identifier
    // with "r#" if it is not a valid identifier.
    syn::parse_str::<Ident>(&name).unwrap_or(format_ident!("r#{}", name))
}

pub fn sanitize_documentation(string: &str) -> Result<String, Error> {
    let arena = Arena::new();
    let node = parse_document(&arena, string, &Default::default());

    for node in node.traverse() {
        let NodeEdge::Start(node) = node else {
            continue;
        };
        let mut ast = node.data.borrow_mut();
        let NodeValue::CodeBlock(block) = &mut ast.value else {
            continue;
        };

        if block.info.is_empty() {
            block.info = "text".into();
        }
    }

    let mut buffer = Vec::with_capacity(string.len());

    format_commonmark(node, &Default::default(), &mut buffer)?;

    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn sanitize_name_with_dot() {
        assert_eq!(sanitize_snake_case_name("foo.bar"), "foo_bar");
    }

    #[test]
    fn sanitize_name_with_dot_and_underscore() {
        assert_eq!(sanitize_snake_case_name("foo.bar_baz"), "foo_bar_baz");
    }

    #[test]
    fn sanitize_reserved_name() {
        assert_eq!(sanitize_snake_case_name("builder"), "_builder");
    }

    #[test]
    fn sanitize_code_block() {
        assert_eq!(
            &sanitize_documentation("```\nfoo\n```\n").unwrap(),
            "``` text\nfoo\n```\n"
        );
    }

    #[test]
    fn sanitize_code_blocks() {
        assert_eq!(
            &sanitize_documentation("```\nfoo\n```\n\n```\nbar\n```\n").unwrap(),
            "``` text\nfoo\n```\n\n``` text\nbar\n```\n"
        );
    }
}