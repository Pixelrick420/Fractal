use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;

fn module_search(module_name: &str, current_file: &str  ) -> io::Result<(Vec<char>, String)> {
     let lib_dir = "";

    if &module_name[0..1] == "\"" {
        let mut file_path = PathBuf::from(module_name.trim().trim_matches('"'));

        match file_path.extension().and_then(|e| e.to_str()) {
            Some("fr") => {}
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported file extension: {}", module_name),
                ))
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
            let file_contents = fs::read_to_string(&canonical_path)?;
            let canonical_str = canonical_path.to_str().unwrap_or("").to_string();
            return Ok((file_contents.chars().collect(), canonical_str));
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Module not found: {}", module_name),
        ))
    } else {
        let lib_path = format!("{}/{}.fr", lib_dir, module_name);
        if Path::new(&lib_path).exists() {
            let contents = fs::read_to_string(&lib_path)?;
            Ok((contents.chars().collect(), lib_path))
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Module not available: {}", module_name),
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

fn print_error(msg: &str) {
    eprintln!("\x1b[1;31mError:\x1b[0m {}", msg);
}

fn print_import_chain(chain: &[String]) {
    eprintln!("\x1b[1;33mImport chain:\x1b[0m");
    for (i, file) in chain.iter().enumerate() {
        eprintln!(
            "  {} \x1b[36m{}\x1b[0m",
            if i == chain.len() - 1 {
                "└─"
            } else {
                "├─"
            },
            file
        );
    }
}

/// Strip the `!start` ... `!end` wrapper from a module's source text.
/// Returns the inner content (everything between `!start` and `!end`).
fn strip_start_end(chars: &[char]) -> Vec<char> {
    let text: String = chars.iter().collect();

    // Find and remove !start (with optional surrounding whitespace / newlines)
    let after_start = if let Some(pos) = text.find("!start") {
        let end = pos + "!start".len();
        // skip a single newline immediately after !start if present
        if text[end..].starts_with('\n') {
            &text[end + 1..]
        } else {
            &text[end..]
        }
    } else {
        &text
    };

    // Find and remove !end
    let before_end = if let Some(pos) = after_start.rfind("!end") {
        // trim a trailing newline before !end if present
        let chunk = &after_start[..pos];
        if chunk.ends_with('\n') {
            &chunk[..chunk.len() - 1]
        } else {
            chunk
        }
    } else {
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

    let mut module_blocks = String::new();
    let mut own_body = String::new();

    while index < chars.len() {
        if chars[index] == '#' {
            if index + 2 < chars.len() && chars[index + 1] == '#' && chars[index + 2] == '#' {
                index += 3;
                while index + 2 < chars.len() {
                    if chars[index] == '#' && chars[index + 1] == '#' && chars[index + 2] == '#' {
                        index += 3;
                        break;
                    }
                    index += 1;
                }
                continue;
            } else {
                while index < chars.len() && chars[index] != '\n' {
                    index += 1;
                }
                continue;
            }
        }

        if chars[index] == '!' {
            let import_word: [char; 6] = ['i', 'm', 'p', 'o', 'r', 't'];
            let mut word_index: usize = 0;
            let mut temp = String::from("!");
            index += 1;

            while index < chars.len() && word_index < 6 && chars[index] == import_word[word_index] {
                temp.push(chars[index]);
                index += 1;
                word_index += 1;
            }

            if word_index == 6 {
                while index < chars.len() && chars[index].is_whitespace() {
                    index += 1;
                }

                let mut module_name = String::new();
                while index < chars.len() && chars[index] != ';' {
                    module_name.push(chars[index]);
                    index += 1;
                }

                if index < chars.len() {
                    index += 1;
                }

                match module_search(&module_name, current_file,) {
                    Ok((module_raw, resolved_path)) => {
                        if import_chain.contains(&resolved_path) {
                            print_error("Circular dependency detected");
                            print_import_chain(import_chain);
                            eprintln!("  \x1b[1;31m└─ {} (circular)\x1b[0m", resolved_path);
                            process::exit(1);
                        }

                        if !visited_modules.contains(&resolved_path) {
                            visited_modules.push(resolved_path.clone());

                            // Strip !start / !end from the imported module before processing
                            let mut module_content = strip_start_end(&module_raw);

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
                                .push_str(&format!("$MODULE_START:{}$\n", extracted_name));
                            this_import_blocks.push_str(&child_own_body);
                            this_import_blocks
                                .push_str(&format!("$MODULE_END:{}$;\n", extracted_name));

                            module_blocks.push_str(&this_import_blocks);
                        }

                        if is_root {
                            own_body.push_str(&module_blocks);
                            module_blocks.clear();
                        }
                    }
                    Err(e) => {
                        print_error(&format!("{}", e));
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