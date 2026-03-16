use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

fn print_error_header(code: &str, title: &str) {
    eprintln!(
        "\x1b[1;31merror[{code}]\x1b[0m\x1b[1m: {title}\x1b[0m",
        code = code,
        title = title
    );
}

fn print_location(file: &str, line: Option<usize>) {
    match line {
        Some(l) => eprintln!(" \x1b[1;34m-->\x1b[0m {file}:{l}"),
        None => eprintln!(" \x1b[1;34m-->\x1b[0m {file}"),
    }
}

fn print_hint(hint: &str) {
    eprintln!(" \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: {hint}");
}

fn print_note(note: &str) {
    eprintln!(" \x1b[1;34m  =\x1b[0m \x1b[1;36mnote\x1b[0m: {note}");
}

fn print_import_chain(chain: &[String]) {
    eprintln!(" \x1b[1;34m  =\x1b[0m \x1b[1;36mnote\x1b[0m: import chain:");
    for (i, file) in chain.iter().enumerate() {
        let connector = if i == chain.len() - 1 {
            "└─"
        } else {
            "├─"
        };
        eprintln!("       {connector} \x1b[36m{file}\x1b[0m");
    }
}

fn blank_line() {
    eprintln!();
}

fn module_search(
    module_name: &str,
    current_file: &str,
    import_line: usize,
) -> io::Result<(Vec<char>, String)> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let lib_dir_path = exe_dir.join("stdlib");
    let lib_dir = lib_dir_path.to_str().unwrap_or("stdlib");

    if module_name.starts_with('"') {
        let mut file_path = PathBuf::from(module_name.trim().trim_matches('"'));

        match file_path.extension().and_then(|e| e.to_str()) {
            Some("fr") => {}
            Some(ext) => {
                print_error_header(
                    "P001",
                    &format!("cannot import file with extension `.{ext}`"),
                );
                print_location(current_file, Some(import_line));
                print_hint(&format!(
                    "only `.fr` source files can be imported — \
                     rename the file to have a `.fr` extension, \
                     or remove `.{ext}` from the import path (the compiler adds `.fr` automatically)"
                ));
                blank_line();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unsupported extension: {ext}"),
                ));
            }
            None => {
                file_path.set_extension("fr");
            }
        }

        let base_path = Path::new(current_file).parent().unwrap_or(Path::new("."));
        let resolved_path = base_path.join(&file_path);
        let canonical_path = resolved_path
            .canonicalize()
            .unwrap_or_else(|_| resolved_path.clone());

        if canonical_path.exists() && canonical_path.is_file() {
            let file_contents = fs::read_to_string(&canonical_path).map_err(|e| {
                let path_str = canonical_path.display().to_string();
                print_error_header(
                    "P002",
                    &format!("failed to read imported file `{path_str}`"),
                );
                print_location(current_file, Some(import_line));
                print_hint(&format!(
                    "the file was found but could not be read — \
                     check that the process has read permission on this file; OS error: {e}"
                ));
                blank_line();
                e
            })?;
            let canonical_str = canonical_path.to_str().unwrap_or("").to_string();
            return Ok((file_contents.chars().collect(), canonical_str));
        }

        let display = canonical_path.display().to_string();
        print_error_header("P003", &format!("imported file not found: `{display}`"));
        print_location(current_file, Some(import_line));
        eprintln!(" \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import {module_name};\x1b[0m");
        eprintln!(
            " \x1b[1;34m  |\x1b[0m          \x1b[1;31m{caret}\x1b[0m no file at this path",
            caret = "^".repeat(module_name.len())
        );
        print_hint(&format!(
            "the path `{display}` does not exist — \
             check for typos in the filename, and note that paths are resolved \
             relative to the importing file, not the working directory"
        ));
        print_note(&format!("searched relative to: `{}`", base_path.display()));
        blank_line();

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("module not found: {module_name}"),
        ))
    } else {
        let lib_path = format!("{lib_dir}/{module_name}.fr");
        if Path::new(&lib_path).exists() {
            let contents = fs::read_to_string(&lib_path).map_err(|e| {
                print_error_header(
                    "P002",
                    &format!("failed to read standard library module `{module_name}`"),
                );
                print_location(current_file, Some(import_line));
                print_hint(&format!(
                    "the module file was found at `{lib_path}` but could not be read — \
                     check file permissions; OS error: {e}"
                ));
                blank_line();
                e
            })?;
            Ok((contents.chars().collect(), lib_path))
        } else {
            print_error_header(
                "P004",
                &format!("unknown standard library module `{module_name}`"),
            );
            print_location(current_file, Some(import_line));
            eprintln!(" \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import {module_name};\x1b[0m");
            eprintln!(
                " \x1b[1;34m  |\x1b[0m          \x1b[1;31m{caret}\x1b[0m not a known standard library module",
                caret = "^".repeat(module_name.len())
            );
            print_hint(&format!(
                "`{module_name}` is not in the standard library — \
                 if this is a local file, quote the path: `!import \"{module_name}.fr\";` \
                 so it is resolved relative to the current file instead"
            ));
            print_note(&format!(
                "standard library modules are looked up in `{lib_dir}/`; \
                 if that directory is wrong, update `lib_dir` in the compiler source"
            ));
            blank_line();

            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("module not available: {module_name}"),
            ))
        }
    }
}

fn get_module_name_from_path(path: &str) -> String {
    Path::new(path.trim_matches('"'))
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn is_standalone_marker(text: &str, pos: usize, marker_len: usize) -> bool {
    if pos > 0 {
        let before = text[..pos].chars().next_back().unwrap_or(' ');
        if !before.is_whitespace() {
            return false;
        }
    }

    let after_pos = pos + marker_len;
    if after_pos < text.len() {
        let after = text[after_pos..].chars().next().unwrap_or(' ');
        if !after.is_whitespace() {
            return false;
        }
    }
    true
}

fn find_standalone(text: &str, marker: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(marker) {
        let pos = search_from + rel;
        if is_standalone_marker(text, pos, marker.len()) {
            return Some(pos);
        }
        search_from = pos + 1;
    }
    None
}

fn rfind_standalone(text: &str, marker: &str) -> Option<usize> {
    let mut last = None;
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find(marker) {
        let pos = search_from + rel;
        if is_standalone_marker(text, pos, marker.len()) {
            last = Some(pos);
        }
        search_from = pos + 1;
    }
    last
}

fn strip_start_end(chars: &[char], source_file: &str) -> Vec<char> {
    let text: String = chars.iter().collect();

    let after_start = if let Some(pos) = find_standalone(&text, "!start") {
        let end = pos + "!start".len();
        if text[end..].starts_with('\n') {
            &text[end + 1..]
        } else {
            &text[end..]
        }
    } else {
        eprintln!(
            "\x1b[1;33mwarning\x1b[0m: \
             `!start` marker not found in `{source_file}`"
        );
        eprintln!(
            " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: \
             every source file should begin with `!start` and end with `!end`"
        );
        eprintln!();
        &text
    };

    let before_end = if let Some(pos) = rfind_standalone(after_start, "!end") {
        let chunk = &after_start[..pos];
        if chunk.ends_with('\n') {
            &chunk[..chunk.len() - 1]
        } else {
            chunk
        }
    } else {
        eprintln!(
            "\x1b[1;33mwarning\x1b[0m: \
             `!end` marker not found in `{source_file}`"
        );
        eprintln!(
            " \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: \
             every source file should begin with `!start` and end with `!end`"
        );
        eprintln!();
        after_start
    };

    before_end.chars().collect()
}

fn traverse(
    chars: &mut Vec<char>,
    visited_modules: &mut Vec<String>,
    current_file: &str,
    import_chain: &mut Vec<String>,
    is_root: bool,
) -> (String, String) {
    let mut index: usize = 0;
    let mut line: usize = 1;

    let mut module_blocks = String::new();
    let mut own_body = String::new();

    own_body.push_str(&format!(
        "$SRCMAP:{}:{}$\n",
        current_file,
        line.saturating_sub(1)
    ));

    while index < chars.len() {
        // Pass string literals through verbatim — # and ! inside strings must not be
        // treated as comments or imports.
        if chars[index] == '"' {
            own_body.push('"');
            index += 1;
            loop {
                if index >= chars.len() || chars[index] == '\n' {
                    // Unterminated string — pass the newline (or EOF) through and let the
                    // lexer produce the proper error.
                    break;
                }
                let c = chars[index];
                own_body.push(c);
                index += 1;
                if c == '\\' && index < chars.len() {
                    // consume the escaped character so a `\"` does not end the string
                    own_body.push(chars[index]);
                    index += 1;
                    continue;
                }
                if c == '"' {
                    break;
                }
            }
            continue;
        }

        if chars[index] == '#' {
            if index + 2 < chars.len() && chars[index + 1] == '#' && chars[index + 2] == '#' {
                index += 3;
                let comment_start_line = line;
                let mut closed = false;
                while index + 2 < chars.len() {
                    if chars[index] == '\n' {
                        line += 1;
                    }
                    if chars[index] == '#' && chars[index + 1] == '#' && chars[index + 2] == '#' {
                        index += 3;
                        closed = true;
                        break;
                    }
                    index += 1;
                }
                if !closed {
                    print_error_header("P005", "unterminated block comment");
                    print_location(current_file, Some(comment_start_line));
                    eprintln!(
                        " \x1b[1;34m  |\x1b[0m   block comment opened here with `###`, \
                         but no matching `###` closing marker was found before end of file"
                    );
                    print_hint("add a closing `###` on its own line to end the block comment");
                    blank_line();
                    process::exit(1);
                }

                own_body.push_str(&format!(
                    "$SRCMAP:{}:{}$\n",
                    current_file,
                    line.saturating_sub(1)
                ));
                continue;
            } else {
                while index < chars.len() && chars[index] != '\n' {
                    index += 1;
                }
                continue;
            }
        }

        if chars[index] == '\n' {
            line += 1;
            own_body.push('\n');
            index += 1;
            continue;
        }

        if chars[index] == '!' {
            let import_word: [char; 6] = ['i', 'm', 'p', 'o', 'r', 't'];
            let mut word_index: usize = 0;
            let mut temp = String::from("!");
            let import_line = line;
            index += 1;

            while index < chars.len() && word_index < 6 && chars[index] == import_word[word_index] {
                temp.push(chars[index]);
                index += 1;
                word_index += 1;
            }

            if word_index == 6 {
                while index < chars.len() && chars[index] == ' ' {
                    index += 1;
                }

                if index >= chars.len() || chars[index] == '\n' || chars[index] == ';' {
                    print_error_header("P006", "`!import` statement has no module name");
                    print_location(current_file, Some(import_line));
                    eprintln!(" \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import;\x1b[0m");
                    eprintln!(" \x1b[1;34m  |\x1b[0m          \x1b[1;31m^ module name missing here\x1b[0m");
                    print_hint(
                        "supply a module name after `!import`: \
                         use `!import \"./path/to/file\";` for a local file \
                         or `!import modname;` for a standard library module",
                    );
                    blank_line();
                    process::exit(1);
                }

                let mut module_name = String::new();
                while index < chars.len() && chars[index] != ';' && chars[index] != '\n' {
                    module_name.push(chars[index]);
                    index += 1;
                }
                let module_name = module_name.trim().to_string();

                if index >= chars.len() || chars[index] == '\n' {
                    print_error_header(
                        "P007",
                        &format!("missing `;` at end of `!import {module_name}`"),
                    );
                    print_location(current_file, Some(import_line));
                    eprintln!(" \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import {module_name}\x1b[0m");
                    eprintln!(
                        " \x1b[1;34m  |\x1b[0m   {pad}\x1b[1;31m^ `;` required here\x1b[0m",
                        pad = " ".repeat(8 + module_name.len())
                    );
                    print_hint(&format!(
                        "every import statement must end with a semicolon — \
                         write it as: `!import {module_name};`"
                    ));
                    blank_line();
                    process::exit(1);
                }

                index += 1;

                if !module_name.starts_with('"')
                    && !module_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_alphabetic() || c == '_')
                {
                    print_error_header("P008", &format!("invalid module name `{module_name}`"));
                    print_location(current_file, Some(import_line));
                    eprintln!(" \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import {module_name};\x1b[0m");
                    eprintln!(
                        " \x1b[1;34m  |\x1b[0m          \x1b[1;31m{caret}\x1b[0m invalid name",
                        caret = "^".repeat(module_name.len())
                    );
                    print_hint(
                        "local file paths must be in double quotes: `!import \"./mymod\";` — \
                         standard library module names must start with a letter or `_`: `!import math;`",
                    );
                    blank_line();
                    process::exit(1);
                }

                match module_search(&module_name, current_file, import_line) {
                    Ok((module_raw, resolved_path)) => {
                        if import_chain.contains(&resolved_path) {
                            let mod_name = get_module_name_from_path(&module_name);
                            print_error_header(
                                "P009",
                                &format!("circular import detected: `{mod_name}` is already being imported"),
                            );
                            print_location(current_file, Some(import_line));
                            eprintln!(
                                " \x1b[1;34m  |\x1b[0m   \x1b[1;31m!import {module_name};\x1b[0m"
                            );
                            eprintln!(
                                " \x1b[1;34m  |\x1b[0m          \x1b[1;31m{caret}\x1b[0m this import creates a cycle",
                                caret = "^".repeat(module_name.len())
                            );
                            print_import_chain(import_chain);
                            eprintln!(
                                "       \x1b[1;31m└─ {resolved_path} ↩ already in the chain above\x1b[0m"
                            );
                            print_hint(
                                "a file cannot directly or transitively import itself — \
                                 extract the shared code into a separate module that neither file imports"
                            );
                            blank_line();
                            process::exit(1);
                        }

                        if !visited_modules.contains(&resolved_path) {
                            visited_modules.push(resolved_path.clone());

                            let mut module_content = strip_start_end(&module_raw, &resolved_path);

                            let extracted_name = get_module_name_from_path(&module_name);

                            import_chain.push(resolved_path.clone());
                            let (child_module_blocks, child_own_body) = traverse(
                                &mut module_content,
                                visited_modules,
                                &resolved_path,
                                import_chain,
                                false,
                            );
                            import_chain.pop();

                            let mut this_import_blocks = String::new();
                            this_import_blocks.push_str(&child_module_blocks);
                            this_import_blocks
                                .push_str(&format!("$MODULE_START:{extracted_name}$\n"));
                            this_import_blocks.push_str(&child_own_body);
                            this_import_blocks
                                .push_str(&format!("$MODULE_END:{extracted_name}$;\n"));

                            module_blocks.push_str(&this_import_blocks);
                        }

                        if is_root {
                            own_body.push_str(&module_blocks);
                            module_blocks.clear();
                        }

                        own_body.push_str(&format!(
                            "$SRCMAP:{}:{}$\n",
                            current_file,
                            line.saturating_sub(1)
                        ));
                    }
                    Err(_) => {
                        process::exit(1);
                    }
                }
            } else {
                own_body.push_str(&temp);
            }
            continue;
        }

        own_body.push(chars[index]);
        index += 1;
    }

    (module_blocks, own_body)
}

pub fn preprocess(program: &str, source_file: &str) -> String {
    let mut chars: Vec<char> = program.chars().collect();
    let mut visited_modules: Vec<String> = Vec::new();
    let mut import_chain = vec![source_file.to_string()];

    let (_empty, output) = traverse(
        &mut chars,
        &mut visited_modules,
        source_file,
        &mut import_chain,
        true,
    );

    output
}
