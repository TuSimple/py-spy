use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Once;

enum CheckMode {
    EXACT,
    ENDWITH,
    CONTAIN,
}

struct Program {
    name: String,
    mode: CheckMode,
}

type Idlelist = HashMap<String, Vec<Program>>;

static mut IDLEFUNCS: Option<Idlelist> = None;
static INIT: Once = Once::new();

/// Load the idle list from file
pub fn load_idle_list(filename: &Option<String>) {
    INIT.call_once(|| {
        // Read the file
        match filename {
            Some(value) => {
                match File::open(value) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        let mut ret = HashMap::new();
                        for line in reader.lines() {
                            match line {
                                Ok(line_string) => {
                                    let items: Vec<&str> = line_string.split_whitespace().collect();
                                    let program_mode = match items[2] {
                                        "X" => Some(CheckMode::EXACT),
                                        "E" => Some(CheckMode::ENDWITH),
                                        "C" => Some(CheckMode::CONTAIN),
                                        _  => None,
                                    };
                                    if let Some(mode) = program_mode {
                                        ret.entry(items[0].to_string()).or_insert(Vec::new()).push(Program{name: items[1].to_string(), mode});
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Load idle list fail! Error: {}", e);
                                    return ;
                                }
                            }
                        }
                        info!("Load idle list succeed!");
                        unsafe {
                            IDLEFUNCS = Some(ret);
                        }
                    }
                    Err(e) => {
                        eprintln!("Load idle list fail! Error: {}", e);
                    }
                }
            }
            None => (),
        }
    });
}

/// Check whether a program is idle.
pub fn check_idle(func: &String, program: &String) -> bool {
    unsafe {
        if let Some(ref list) = IDLEFUNCS {
            match list.get(func) {
                Some(value) => {
                    let mut ret: bool = false;
                    for item in value.iter() {
                        ret = match item.mode {
                            CheckMode::EXACT => *program == *item.name,
                            CheckMode::ENDWITH => program.ends_with(&*item.name),
                            CheckMode::CONTAIN => program.contains(&*item.name),
                        };
                        if ret {
                            break;
                        }
                    }
                    ret
                },
                None => false,
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_insertion() {
        // Create the file named "test_idle_list" before running this test with following content
        // func_a program_a X
        // func_a program_b E
        // func_b program_a C
        // func_c program_b E
        let filename = String::from("tests/test_idle_list");
        load_idle_list(&Some(filename));

        // load success
        unsafe {
            assert!(IDLEFUNCS.is_some());
        }

        let func_a_string = String::from("func_a");
        let func_b_string = String::from("func_b");
        let func_c_string = String::from("func_c");
        let func_d_string = String::from("func_d");
        let program_a_string = String::from("program_a");
        let program_a_contain = String::from("xxx_program_a_xxx");
        let program_a_ncontain = String::from("xxx_program_xxx_a");
        let program_b_string = String::from("program_b");
        let program_b_endwith = String::from("xxx_program_b");
        let program_b_nendwith = String::from("xxx_program_b?");

        // Check cases
        assert_eq!(check_idle(&func_a_string, &program_a_string), true);
        assert_eq!(check_idle(&func_a_string, &program_a_contain), false);
        assert_eq!(check_idle(&func_a_string, &program_b_string), true);
        assert_eq!(check_idle(&func_a_string, &program_b_endwith), true);
        assert_eq!(check_idle(&func_a_string, &program_b_nendwith), false);
        assert_eq!(check_idle(&func_b_string, &program_a_string), true);
        assert_eq!(check_idle(&func_b_string, &program_a_contain), true);
        assert_eq!(check_idle(&func_b_string, &program_a_ncontain), false);
        assert_eq!(check_idle(&func_c_string, &program_b_string), true);
        assert_eq!(check_idle(&func_c_string, &program_b_endwith), true);
        assert_eq!(check_idle(&func_c_string, &program_b_nendwith), false);
        assert_eq!(check_idle(&func_d_string, &program_a_string), false);
    }
}
