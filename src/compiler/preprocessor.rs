use std::fmt::format;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Command;

fn module_search(
    module_name: &str,
    contents: &mut String,
    current_file: &str,
) -> io::Result<(Vec<char>, String)> {
    let libs = ["sample_lib"];

    if (&module_name[0..1] == "\"") {
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
            let module_name_extracted = canonical_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            let file_contents = fs::read_to_string(&canonical_path)?;
            let canonical_str = canonical_path.to_str().unwrap_or("").to_string();

            return Ok((file_contents.chars().collect(), canonical_str));
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Module not found: {}", module_name),
        ))
    } else if libs.contains(&module_name) {
        let lib_path = format!("lib/{}.fr", module_name);
        let contents = fs::read_to_string(&lib_path)?;
        Ok((contents.chars().collect(), lib_path))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Module not available: {}", module_name),
        ))
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

fn print_import_chain(chain: &Vec<String>) {
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

fn traverse(
    chars: &mut Vec<char>,
    visited_modules: &mut Vec<String>,
    contents: &mut String,
    current_file: &str,
    import_chain: &mut Vec<String>,
) {
    let mut index: usize = 0;

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
        } else if chars[index] == '!' {
            let mut temp = String::from("!");
            let word: Vec<char> = vec!['i', 'm', 'p', 'o', 'r', 't'];
            let mut word_index: usize = 0;
            index += 1;
            while index < chars.len() && word_index < 6 && chars[index] == word[word_index] {
                temp.push(chars[index]);
                index += 1;
                word_index += 1;
            }
            if word_index == 6 {
                let mut module_name = String::from("");
                while index < chars.len() && chars[index].is_whitespace() {
                    index += 1;
                }

                while index < chars.len() && chars[index] != ';' {
                    module_name.push(chars[index]);
                    index += 1;
                }

                match module_search(&module_name, contents, current_file) {
                    Ok((mut module_content, resolved_path)) => {
                        if import_chain.contains(&resolved_path) {
                            print_error(&format!("Circular dependency detected"));
                            print_import_chain(import_chain);
                            eprintln!("  \x1b[1;31m└─ {} (circular)\x1b[0m", resolved_path);
                            process::exit(1);
                        }

                        if !visited_modules.contains(&resolved_path) {
                            visited_modules.push(resolved_path.clone());

                            let extracted_name = get_module_name_from_path(&module_name);
                            let start_marker = format!("$MODULE_START:{}$\n", extracted_name);
                            let end_marker = format!("$MODULE_END:{}$", extracted_name);

                            contents.push_str(&start_marker);

                            import_chain.push(resolved_path.clone());
                            traverse(
                                &mut module_content,
                                visited_modules,
                                contents,
                                &resolved_path,
                                import_chain,
                            );
                            import_chain.pop();

                            contents.push_str(&end_marker);
                        }
                    }
                    Err(e) => {
                        print_error(&format!("{}", e));
                        process::exit(1);
                    }
                }
            } else {
                contents.push_str(&temp);
            }
        } else {
            if index >= chars.len() {
                break;
            }
            contents.push(chars[index]);
            index += 1;
        }
    }
}

pub fn preprocess(program: &str, source_file: &str) -> String {
    let mut chars: Vec<char> = program.chars().collect();
    let mut contents = String::from("");
    let mut visited_modules = Vec::new();
    let mut import_chain = vec![source_file.to_string()];
    traverse(
        &mut chars,
        &mut visited_modules,
        &mut contents,
        source_file,
        &mut import_chain,
    );
    return contents;
}
