use std::vec;
use tower_lsp::lsp_types::*;
use tree_sitter::{Node, Point};

pub fn point_to_position(point: Point) -> Position {
    return Position {
        line: point.row as u32,
        character: point.column as u32,
    };
}

pub fn node_to_location(node: Node, uri: &Url) -> Location {
    return Location {
        uri: uri.to_owned(),
        range: Range {
            start: point_to_position(node.start_position()),
            end: point_to_position(node.end_position()),
        },
    };
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

pub fn extract_word_at(text: &str, position: Position) -> String {
    let line = text.lines().nth(position.line as usize).unwrap_or("");

    let bytes = line.as_bytes();
    let mut start = position.character as usize;
    let mut end = position.character as usize;

    // Move backward to find word start
    while start > 0 && is_word_char(bytes[start - 1]) {
        start -= 1;
    }

    // Move forward to find word end
    while end < bytes.len() && is_word_char(bytes[end]) {
        end += 1;
    }

    line[start..end].to_string()
}

fn extract_single_ident(line: &str, pos: &mut usize) -> String {
    let bytes = line.as_bytes();

    let end = *pos;
    while *pos > 0 && is_word_char(bytes[*pos - 1]) {
        *pos -= 1;
    }
    return line[*pos..end].to_string();
}

fn extract_array_access(line: &str, pos: &mut usize) -> bool {
    let bytes = line.as_bytes();
    if *pos <= 0 || bytes[*pos - 1] != b']' {
        return false;
    }

    let mut array_balance = 0; // closing - opening
    while *pos > 0 {
        *pos -= 1;
        if bytes[*pos] == b']' {
            array_balance += 1;
        }
        if bytes[*pos] == b'[' {
            array_balance -= 1;
        }
        if array_balance == 0 {
            return true;
        }
    }
    return false;
}

// return the list of identifiers and array accesses that exist before a '.' in the specified line and position, returns reversed sequence
pub fn extract_identifier_sequence(text: &str, position: Position) -> Vec<String> {
    let line = text.lines().nth(position.line as usize).unwrap_or("");
    let mut idents = vec![];

    let bytes = line.as_bytes();
    let mut start = position.character as usize;

    extract_single_ident(line, &mut start); // removes the leading ident
    while start > 0 && bytes[start - 1] == b'.' {
        start -= 1;
        while extract_array_access(line, &mut start) {
            idents.push("[]".to_string());
        }
        idents.push(extract_single_ident(line, &mut start));
    }
    return idents;
}
