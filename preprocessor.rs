use std::fs;
use std::io;
use std::process;
use std::process::Command;
use std::path::Path;
use std::path::PathBuf;

fn module_search(module_name: &str) -> io::Result<String> {

    let libs = ["sample_lib"];


    if (&module_name[0..1]=="\""){
       let mut file_path = PathBuf::from(
                module_name.trim().trim_matches('"')
            );

           match file_path.extension().and_then(|e| e.to_str()) {
            Some("rs")=>{}
            Some(_)=>{
                 return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    ("Unsupported file extension"),
                ))
            }
            None=>{
                file_path.set_extension("rs");
            }
           }
        let path = Path::new(&file_path);

        if path.exists() && path.is_file(){
            println!("File exists");
            return std::fs::read_to_string(file_path);
            
            
        }
           
            Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", module_name),
        ))
        
        
    }
    else if libs.contains(&module_name) {
        std::fs::read_to_string(format!("lib/{}.rs", module_name))
    } else {

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not available", module_name),
        ))
    }
}


pub fn preprocess(program: &str) -> String {
    let chars: Vec<char> = program.chars().collect();
    let mut index: usize = 0;
    let mut contents = String::from("");
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
        }else if chars[index] == '!'{
            let mut temp = String::from("!");
            let word: Vec<char> = vec!['i','m','p','o','r','t'];
            let mut word_index: usize = 0;
            index += 1;
            while index<chars.len()  && word_index < 6 && chars[index] == word[word_index]{
                temp.push(chars[index]);
                index += 1;
                word_index += 1;
            }
            if word_index == 6{
                let mut module_name = String::from("");
                while index<chars.len() && chars[index].is_whitespace(){
                    index += 1;
                }
                
                while index<chars.len() && chars[index]!=';'{
                    module_name.push(chars[index]);
                    index += 1;
                }
              
                let module_content = module_search(&module_name); 
                match module_search(&module_name) {
                Ok(module_content) => contents.push_str(&module_content),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
               
                
            }else{
                contents.push_str(&temp);
            }
        }else {
            if index >= chars.len(){
                break;
            }
            contents.push(chars[index]);
            index += 1;
        }
    }
    return contents;
}