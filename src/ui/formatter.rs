pub fn format_code(src: &str) -> String {
    let physical: Vec<&str> = src.lines().collect();

    let normalised: Vec<String> = physical.iter().map(|l| normalise_line(l.trim())).collect();

    let mut joined: Vec<String> = Vec::with_capacity(normalised.len());
    for line in &normalised {
        let t = line.trim();
        if t == "{" && !joined.is_empty() {
            let prev = joined.pop().unwrap();
            if prev.trim().is_empty() {
                joined.push(prev);
                joined.push(line.clone());
            } else {
                joined.push(format!("{} {{", prev.trim_end()));
            }
        } else {
            joined.push(line.clone());
        }
    }

    let mut output = String::with_capacity(src.len() + 128);
    let mut depth: i32 = 0;
    let mut prev_blank = false;
    let mut blank_count = 0u32;

    for line in &joined {
        let t = line.trim();

        if t.is_empty() {
            blank_count += 1;
            if blank_count == 1 && !prev_blank {
                output.push('\n');
                prev_blank = true;
            }
            continue;
        }

        blank_count = 0;
        prev_blank = false;

        let (opens, closes) = count_braces(t);

        if t == "!start" {
            let indent = "    ".repeat(depth as usize);
            output.push_str(&indent);
            output.push_str(t);
            output.push('\n');
            depth += 1;
            continue;
        }

        if t == "!end" {
            depth = (depth - 1).max(0);
            let indent = "    ".repeat(depth as usize);
            output.push_str(&indent);
            output.push_str(t);
            output.push('\n');
            continue;
        }

        let effective_depth = if t.starts_with('}') {
            let d = (depth - closes).max(0);
            depth = d + opens;
            d
        } else {
            let d = depth;
            depth = (depth + opens - closes).max(0);
            d
        };

        let indent = "    ".repeat(effective_depth as usize);
        output.push_str(&indent);
        output.push_str(t);
        output.push('\n');
    }

    let result = output.trim_end_matches('\n').to_string();
    if result.is_empty() {
        return String::new();
    }
    result + "\n"
}

fn count_braces(line: &str) -> (i32, i32) {
    let mut opens = 0i32;
    let mut closes = 0i32;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '"' => {
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                    } else if chars[i] == '"' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            '\'' => {
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' {
                        i += 2;
                    } else if chars[i] == '\'' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            '#' => break,
            '{' => {
                opens += 1;
                i += 1;
            }
            '}' => {
                closes += 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    (opens, closes)
}

fn ends_with_type_keyword(out: &str) -> bool {
    let s = out.trim_end_matches(' ');
    matches!(
        s.rsplit_once(':').map(|(_, kw)| kw),
        Some("array") | Some("list") | Some("struct")
    )
}

fn normalise_line(line: &str) -> String {
    if line.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len() + 16);
    let mut i = 0;

    let mut angle_depth: u32 = 0;

    while i < chars.len() {
        if chars[i] == '"' {
            ensure_space_before(&mut out);
            out.push('"');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                } else if chars[i] == '"' {
                    out.push('"');
                    i += 1;
                    break;
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }

        if chars[i] == '\'' {
            out.push('\'');
            i += 1;
            if i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            if i < chars.len() && chars[i] == '\'' {
                out.push('\'');
                i += 1;
            }
            continue;
        }

        if chars[i] == '#' {
            let trimmed_out = out.trim_end().to_string();
            if !trimmed_out.is_empty() {
                out = trimmed_out;
                out.push(' ');
            }
            while i < chars.len() {
                out.push(chars[i]);
                i += 1;
            }
            break;
        }

        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if chars[i] == ';' {
            out.push(';');
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() && chars[i] != '#' {
                out.push(' ');
            }
            continue;
        }

        if chars[i] == '{' || chars[i] == '}' {
            ensure_space_before(&mut out);
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == ',' {
            out.push(',');
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() {
                out.push(' ');
            }
            continue;
        }

        if chars[i] == ':' && i + 1 < chars.len() && chars[i + 1] == ':' {
            ensure_space_before(&mut out);
            out.push_str("::");
            i += 2;
            continue;
        }

        if chars[i] == '!' {
            ensure_space_before(&mut out);
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let kw: String = chars[start..i].iter().collect();
            out.push_str(&kw);

            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }

            if i < chars.len() && chars[i] == '(' {
                let bare = &kw[1..];
                if matches!(bare, "if" | "else" | "for" | "while" | "func" | "return") {
                    out.push(' ');
                }
            }
            continue;
        }

        if chars[i] == ':' {
            ensure_space_before(&mut out);
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ty: String = chars[start..i].iter().collect();
            out.push_str(&ty);

            continue;
        }

        if i + 1 < chars.len() {
            let two: String = chars[i..i + 2].iter().collect();
            if two == "&&" || two == "||" {
                ensure_space_before(&mut out);
                out.push_str(&two);
                i += 2;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                out.push(' ');
                continue;
            }
        }

        if i + 1 < chars.len() {
            let two: String = chars[i..i + 2].iter().collect();
            if is_two_char_op(&two) {
                ensure_space_before(&mut out);
                out.push_str(&two);
                i += 2;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                out.push(' ');
                continue;
            }
        }

        if chars[i] == '<' {
            if ends_with_type_keyword(&out) || angle_depth > 0 {
                angle_depth += 1;
                out.push('<');
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
            } else {
                ensure_space_before(&mut out);
                out.push('<');
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                out.push(' ');
            }
            continue;
        }

        if chars[i] == '>' {
            if angle_depth > 0 {
                if out.ends_with(' ') {
                    out.pop();
                }
                angle_depth -= 1;
                out.push('>');
                i += 1;
            } else {
                ensure_space_before(&mut out);
                out.push('>');
                i += 1;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                if i < chars.len() {
                    out.push(' ');
                }
            }
            continue;
        }

        if is_binary_op_char(chars[i]) {
            ensure_space_before(&mut out);
            out.push(chars[i]);
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            if i < chars.len() {
                out.push(' ');
            }
            continue;
        }

        if matches!(chars[i], '(' | ')' | '[' | ']') {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let start = i;
        while i < chars.len()
            && !chars[i].is_whitespace()
            && !is_binary_op_char(chars[i])
            && !matches!(
                chars[i],
                '"' | '\''
                    | '#'
                    | '!'
                    | ':'
                    | ','
                    | ';'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
            )
        {
            i += 1;
        }
        if i > start {
            ensure_space_before(&mut out);
            let token: String = chars[start..i].iter().collect();
            out.push_str(&token);
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out.trim_end().to_string()
}

fn ensure_space_before(out: &mut String) {
    if let Some(last) = out.chars().last() {
        if last != ' ' && last != '(' && last != '[' && last != '<' {
            out.push(' ');
        }
    }
}

fn is_two_char_op(s: &str) -> bool {
    matches!(
        s,
        "->" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | ">=" | "<=" | "==" | "~="
    )
}

fn is_binary_op_char(c: char) -> bool {
    matches!(c, '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' | '=')
}
