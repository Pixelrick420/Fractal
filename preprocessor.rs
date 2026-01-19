use std::fs;
use std::io;
use std::process;
use std::process::Command;
use std::path::Path;
use std::path::PathBuf;

fn module_search(module_name: &str) -> io::Result<Vec<char>> {

    let libs = ["sample_lib"];


    if (&module_name[0..1]=="\""){
       let mut file_path = PathBuf::from(
                module_name.trim().trim_matches('"')
            );

           match file_path.extension().and_then(|e| e.to_str()) {
            Some("fr")=>{}
            Some(_)=>{
                 return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported file extension {:?}",module_name),
                ))
            }
            None=>{
                file_path.set_extension("fr");
            }
           }
        let path = Path::new(&file_path);

        if path.exists() && path.is_file(){
            println!("File exists {}",module_name);
            let contents = fs::read_to_string(file_path)?;
            return Ok(contents.chars().collect());
            
            
        }
           
            Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", module_name),
        ))
        
        
    }
    else if libs.contains(&module_name) {
        let contents = fs::read_to_string(format!("lib/{}.fr", module_name))?;
        Ok(contents.chars().collect())
    } else {

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not available", module_name),
        ))
    }
}

fn traverse(chars: &mut Vec <char>, visited_modules: &mut Vec<String>, contents: &mut String){
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
                Ok(mut module_content) => {
                    if !visited_modules.contains(&module_name){
                       
                        visited_modules.push(module_name);
                        traverse(&mut module_content, visited_modules,contents);
                    }
                    
                },
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
    
    

}
pub fn preprocess(program: &str) -> String {
    let mut chars: Vec<char> = program.chars().collect();
    let mut contents = String::from("");
    let mut visited_modules =Vec::new();
    traverse(&mut chars, &mut visited_modules, &mut contents);
    return contents;
    
}