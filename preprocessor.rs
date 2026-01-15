
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
        } else {
            if index >= chars.len(){
                break;
            }
            contents.push(chars[index]);
            index += 1;
        }
    }
    return contents;
}