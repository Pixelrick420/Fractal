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

    let mut after_scope: bool = false;

    while i < chars.len() {
        let was_after_scope = after_scope;
        after_scope = false;

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
            after_scope = was_after_scope;
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
            out.push_str("::");
            i += 2;
            after_scope = true;
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
            if !was_after_scope {
                ensure_space_before(&mut out);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_basic() {
        assert_eq!(normalise_line("math::pi"), "math::pi");
    }

    #[test]
    fn test_scope_spaces_around() {
        assert_eq!(normalise_line("std   ::   io"), "std::io");
    }

    #[test]
    fn test_scope_chained() {
        assert_eq!(normalise_line("a::b::c"), "a::b::c");
    }

    #[test]
    fn test_scope_in_assignment() {
        assert_eq!(normalise_line("x = math::pi"), "x = math::pi");
    }

    #[test]
    fn test_scope_as_rhs_of_type() {
        assert_eq!(normalise_line(":int b = math::pi;"), ":int b = math::pi;");
    }

    #[test]
    fn test_scope_in_expression() {
        assert_eq!(normalise_line("x = a + math::pi"), "x = a + math::pi");
    }

    #[test]
    fn test_type_int() {
        assert_eq!(normalise_line(":int a = 4;"), ":int a = 4;");
    }

    #[test]
    fn test_type_float() {
        assert_eq!(normalise_line(":float b = 5.0;"), ":float b = 5.0;");
    }

    #[test]
    fn test_type_boolean() {
        assert_eq!(
            normalise_line(":boolean flag = true;"),
            ":boolean flag = true;"
        );
    }

    #[test]
    fn test_type_char() {
        assert_eq!(normalise_line(":char c = 'x';"), ":char c = 'x';");
    }

    #[test]
    fn test_type_array_with_size() {
        assert_eq!(normalise_line(":array<int,5> arr"), ":array<int, 5> arr");
    }

    #[test]
    fn test_type_array_char() {
        assert_eq!(
            normalise_line(":array<char,10> string = \"hello 1234\";"),
            ":array<char, 10> string = \"hello 1234\";"
        );
    }

    #[test]
    fn test_type_list() {
        assert_eq!(normalise_line(":list<int> nums"), ":list<int> nums");
    }

    #[test]
    fn test_type_struct_generic() {
        assert_eq!(normalise_line(":struct<MYSTRUCT>"), ":struct<MYSTRUCT>");
    }

    #[test]
    fn test_type_struct_self_ref() {
        assert_eq!(
            normalise_line(":struct<STRUCTNAME> next;"),
            ":struct<STRUCTNAME> next;"
        );
    }

    #[test]
    fn test_type_in_func_param() {
        assert_eq!(normalise_line(":int a, :float b"), ":int a, :float b");
    }

    #[test]
    fn test_type_annotation_extra_spaces() {
        assert_eq!(normalise_line(":int    a   =   4  ;"), ":int a = 4;");
    }

    #[test]
    fn test_keyword_start() {
        assert_eq!(normalise_line("!start"), "!start");
    }

    #[test]
    fn test_keyword_end() {
        assert_eq!(normalise_line("!end"), "!end");
    }

    #[test]
    fn test_keyword_import() {
        assert_eq!(normalise_line("!import \"math\";"), "!import \"math\";");
    }

    #[test]
    fn test_keyword_if() {
        assert_eq!(normalise_line("!if(a > b)"), "!if (a > b)");
    }

    #[test]
    fn test_keyword_if_spaces() {
        assert_eq!(normalise_line("!if  (  a > b  )"), "!if (a > b)");
    }

    #[test]
    fn test_keyword_else() {
        assert_eq!(normalise_line("!else"), "!else");
    }

    #[test]
    fn test_keyword_while() {
        assert_eq!(normalise_line("!while (a > b)"), "!while (a > b)");
    }

    #[test]
    fn test_keyword_for() {
        assert_eq!(
            normalise_line("!for (:int i, 0, n, 1)"),
            "!for (:int i, 0, n, 1)"
        );
    }

    #[test]
    fn test_keyword_return() {
        assert_eq!(normalise_line("!return (a > b);"), "!return (a > b);");
    }

    #[test]
    fn test_keyword_func_signature() {
        assert_eq!(
            normalise_line("!func hello(:int a, :float b) -> :bool"),
            "!func hello(:int a, :float b) -> :bool"
        );
    }

    #[test]
    fn test_keyword_break() {
        assert_eq!(normalise_line("!break;"), "!break;");
    }

    #[test]
    fn test_keyword_continue() {
        assert_eq!(normalise_line("!continue;"), "!continue;");
    }

    #[test]
    fn test_comment_standalone() {
        assert_eq!(normalise_line("# this is a comment"), "# this is a comment");
    }

    #[test]
    fn test_comment_inline() {
        assert_eq!(
            normalise_line(":int a = 4; # comment"),
            ":int a = 4; # comment"
        );
    }

    #[test]
    fn test_comment_inline_extra_spaces() {
        assert_eq!(normalise_line("a = 1;    # note"), "a = 1; # note");
    }

    #[test]
    fn test_comment_with_braces() {
        assert_eq!(
            normalise_line("# {} acts as placeholder"),
            "# {} acts as placeholder"
        );
    }

    #[test]
    fn test_comment_for_loop_inline() {
        assert_eq!(
            normalise_line("!for (:int i, 0, n, 1) {     # variable, start, stop, step"),
            "!for (:int i, 0, n, 1) { # variable, start, stop, step"
        );
    }

    #[test]
    fn test_op_plus_equals() {
        assert_eq!(normalise_line("a+=1"), "a += 1");
    }

    #[test]
    fn test_op_minus_equals() {
        assert_eq!(normalise_line("a -= 1;"), "a -= 1;");
    }

    #[test]
    fn test_op_star_equals() {
        assert_eq!(normalise_line("a*=2"), "a *= 2");
    }

    #[test]
    fn test_op_slash_equals() {
        assert_eq!(normalise_line("a/=2"), "a /= 2");
    }

    #[test]
    fn test_op_equals_equals() {
        assert_eq!(normalise_line("a==b"), "a == b");
    }

    #[test]
    fn test_op_tilde_equals() {
        assert_eq!(normalise_line("a~=b"), "a ~= b");
    }

    #[test]
    fn test_op_greater_equals() {
        assert_eq!(normalise_line("a>=b"), "a >= b");
    }

    #[test]
    fn test_op_less_equals() {
        assert_eq!(normalise_line("a<=b"), "a <= b");
    }

    #[test]
    fn test_op_arrow() {
        assert_eq!(normalise_line("!func f() -> :int"), "!func f() -> :int");
    }

    #[test]
    fn test_op_and() {
        assert_eq!(normalise_line("a&&b"), "a && b");
    }

    #[test]
    fn test_op_or() {
        assert_eq!(normalise_line("a||b"), "a || b");
    }

    #[test]
    fn test_op_greater() {
        assert_eq!(normalise_line("a > b"), "a > b");
    }

    #[test]
    fn test_op_less() {
        assert_eq!(normalise_line("a < b"), "a < b");
    }

    #[test]
    fn test_op_plus() {
        assert_eq!(normalise_line("a + 1"), "a + 1");
    }

    #[test]
    fn test_op_minus() {
        assert_eq!(normalise_line("a - 1"), "a - 1");
    }

    #[test]
    fn test_string_basic() {
        assert_eq!(normalise_line("\"hello 1234\""), "\"hello 1234\"");
    }

    #[test]
    fn test_string_with_format_braces() {
        assert_eq!(
            normalise_line("print(\"{}, {}, {}\", a, b, c);"),
            "print(\"{}, {}, {}\", a, b, c);"
        );
    }

    #[test]
    fn test_string_with_escape() {
        assert_eq!(
            normalise_line("print(\"Hello\\n\");"),
            "print(\"Hello\\n\");"
        );
    }

    #[test]
    fn test_string_assignment() {
        assert_eq!(
            normalise_line(":array<char,10> string = \"hello 1234\";"),
            ":array<char, 10> string = \"hello 1234\";"
        );
    }

    #[test]
    fn test_char_literal() {
        assert_eq!(normalise_line(":char c = 'x';"), ":char c = 'x';");
    }

    #[test]
    fn test_char_literal_escape() {
        assert_eq!(normalise_line(":char nl = '\\n';"), ":char nl = '\\n';");
    }

    #[test]
    fn test_func_call_single_arg() {
        assert_eq!(normalise_line("print(\"Hello\");"), "print(\"Hello\");");
    }

    #[test]
    fn test_func_call_two_args() {
        assert_eq!(normalise_line("append(nums, a);"), "append(nums, a);");
    }

    #[test]
    fn test_func_call_cast_arg() {
        assert_eq!(
            normalise_line("append(arr, (int) b);"),
            "append(arr, (int) b);"
        );
    }

    #[test]
    fn test_func_call_pop() {
        assert_eq!(normalise_line(":int c = pop(arr);"), ":int c = pop(arr);");
    }

    #[test]
    fn test_expr_parens() {
        assert_eq!(normalise_line("a = (a + 1);"), "a = (a + 1);");
    }

    #[test]
    fn test_expr_cast_float() {
        assert_eq!(
            normalise_line("b = (b + (float) 1);"),
            "b = (b + (float) 1);"
        );
    }

    #[test]
    fn test_expr_comparison_in_parens() {
        assert_eq!(normalise_line("!return (a > b);"), "!return (a > b);");
    }

    #[test]
    fn test_struct_field_int() {
        assert_eq!(normalise_line(":int a;"), ":int a;");
    }

    #[test]
    fn test_struct_field_array() {
        assert_eq!(
            normalise_line(":array<int,10> arr;"),
            ":array<int, 10> arr;"
        );
    }

    #[test]
    fn test_struct_field_with_comment() {
        assert_eq!(
            normalise_line(":struct<STRUCTNAME> next; # could be self reference"),
            ":struct<STRUCTNAME> next; # could be self reference"
        );
    }

    #[test]
    fn test_format_brace_joins_to_prev_line() {
        let src = "!if(a > b)\n{\n    a = 1;\n}";
        let result = format_code(src);
        assert!(result.contains("!if (a > b) {"), "got: {}", result);
    }

    #[test]
    fn test_format_start_end_depth() {
        let src = "!start\n:int x = 1;\n!end";
        let result = format_code(src);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "!start");
        assert_eq!(lines[1], "    :int x = 1;");
        assert_eq!(lines[2], "!end");
    }

    #[test]
    fn test_format_nested_block_indentation() {
        let src = "!start\n!if(a > b) {\na = 1;\n}\n!end";
        let result = format_code(src);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "!start");
        assert_eq!(lines[1], "    !if (a > b) {");
        assert_eq!(lines[2], "        a = 1;");
        assert_eq!(lines[3], "    }");
        assert_eq!(lines[4], "!end");
    }

    #[test]
    fn test_format_blank_lines_collapsed() {
        let src = "!start\n\n\n:int x = 1;\n!end";
        let result = format_code(src);

        assert!(!result.contains("\n\n\n"), "got: {}", result);
    }

    #[test]
    fn test_format_func_brace_on_same_line() {
        let src = "!func hello(:int a) -> :bool\n{\n!return true;\n}";
        let result = format_code(src);
        assert!(
            result.contains("!func hello(:int a) -> :bool {"),
            "got: {}",
            result
        );
    }

    #[test]
    fn test_format_string_braces_not_counted() {
        let src = "!start\nprint(\"{}\", x);\n!end";
        let result = format_code(src);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[1], "    print(\"{}\", x);");
    }
}
