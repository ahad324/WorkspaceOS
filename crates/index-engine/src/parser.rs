use tree_sitter::{Node, Parser};

#[derive(Debug, Clone)]
pub struct ParsedSymbol {
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

pub fn extract_symbols(language: &str, code: &str) -> Vec<ParsedSymbol> {
    let mut parser = Parser::new();
    let lang = match language {
        "rust" => tree_sitter_rust::language(),
        "typescript" | "tsx" | "javascript" => tree_sitter_typescript::language_typescript(),
        _ => return Vec::new(),
    };

    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }

    let tree = match parser.parse(code, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut symbols = Vec::new();
    traverse_nodes(tree.root_node(), code, language, &mut symbols);
    symbols
}

fn traverse_nodes(node: Node, code: &str, language: &str, symbols: &mut Vec<ParsedSymbol>) {
    let node_type = node.kind();

    let symbol_kind = match (language, node_type) {
        // Rust rules
        ("rust", "function_item") => Some("function"),
        ("rust", "struct_item") => Some("struct"),
        ("rust", "enum_item") => Some("enum"),
        ("rust", "trait_item") => Some("trait"),
        ("rust", "mod_item") => Some("module"),
        ("rust", "impl_item") => Some("impl"),

        // TypeScript/JavaScript rules
        (_, "function_declaration") => Some("function"),
        (_, "class_declaration") => Some("class"),
        (_, "interface_declaration") => Some("interface"),
        (_, "enum_declaration") => Some("enum"),
        (_, "type_alias_declaration") => Some("type"),

        _ => None,
    };

    if let Some(kind) = symbol_kind {
        // Retrieve name of symbol using AST name field
        let name_node = node.child_by_field_name("name");
        let name = if let Some(n_node) = name_node {
            let range = n_node.range();
            if range.end_byte <= code.len() && range.start_byte < range.end_byte {
                code[range.start_byte..range.end_byte].to_string()
            } else {
                String::new()
            }
        } else if kind == "impl" {
            // For Rust impl items, there might not be a direct 'name' child.
            // We can grab the type child instead:
            let type_node = node.child_by_field_name("type");
            if let Some(t_node) = type_node {
                let range = t_node.range();
                code[range.start_byte..range.end_byte].to_string()
            } else {
                "impl".to_string()
            }
        } else {
            String::new()
        };

        if !name.is_empty() {
            let start = node.start_position();
            let end = node.end_position();
            symbols.push(ParsedSymbol {
                name,
                kind: kind.to_string(),
                start_line: start.row + 1, // 1-indexed line
                start_column: start.column,
                end_line: end.row + 1,
                end_column: end.column,
            });
        }
    }

    // Traverse children recursively
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            traverse_nodes(cursor.node(), code, language, symbols);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

pub fn extract_imports(language: &str, code: &str) -> Vec<String> {
    let mut parser = Parser::new();
    let lang = match language {
        "rust" => tree_sitter_rust::language(),
        "typescript" | "tsx" | "javascript" => tree_sitter_typescript::language_typescript(),
        _ => return Vec::new(),
    };

    if parser.set_language(&lang).is_err() {
        return Vec::new();
    }

    let tree = match parser.parse(code, None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let mut imports = Vec::new();
    traverse_imports(tree.root_node(), code, language, &mut imports);
    imports
}

fn traverse_imports(node: Node, code: &str, language: &str, imports: &mut Vec<String>) {
    let node_type = node.kind();
    match (language, node_type) {
        ("rust", "use_declaration") => {
            let range = node.range();
            let mut text = code[range.start_byte..range.end_byte].trim().to_string();
            if text.starts_with("use ") {
                text = text["use ".len()..].to_string();
            }
            if text.ends_with(';') {
                text.pop();
            }
            let cleaned = text.trim().to_string();
            if !cleaned.is_empty() {
                imports.push(cleaned);
            }
        }
        (_, "import_statement") => {
            let source_node = node.child_by_field_name("source");
            if let Some(s_node) = source_node {
                let range = s_node.range();
                let text = code[range.start_byte..range.end_byte].trim().to_string();
                let cleaned = text
                    .trim_matches(|c| c == '\'' || c == '"' || c == '`')
                    .to_string();
                if !cleaned.is_empty() {
                    imports.push(cleaned);
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            traverse_imports(cursor.node(), code, language, imports);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
