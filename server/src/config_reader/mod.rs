use linked_hash_map::LinkedHashMap;
use signal::Signal;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

extern crate yaml_rust;

use std::borrow::Borrow;
use yaml_rust::{Yaml, YamlLoader};

pub struct Task {
    program_name: String,
    program_path: PathBuf,
    numprocs: u16,
    umask: u16,
    woking_dir: PathBuf,
    autostart: bool,
    autorestart: bool,
    exitcodes: Vec<u8>,
    startretries: u16,
    starttime: u16,
    stopsignal: Signal,
}

pub struct TaskList(Vec<Task>);

struct ReturnTypesForGetValByKey {}

impl TaskList {}

trait GetValByKey {
    fn get_val_by_key<'a>(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<Self>
    where
        Self: Sized;
}

impl GetValByKey for String {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<String> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_str() {
                Some(b) => Some(String::from(b)),
                None => {
                    eprintln!(
                        "Error parsing {:#?} as string for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for programm {} is not found", key, prog_name);
                None
            }
        }
    }
}

impl GetValByKey for u32 {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<u32> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_i64() {
                Some(b) => match u32::try_from(b) {
                    Ok(c) => Some(c),
                    Err(_) => {
                        eprintln!(
                            "Error parsing {:#?} as uint32 for {} field in {}",
                            b, key, prog_name,
                        );
                        None
                    }
                },
                None => {
                    eprintln!(
                        "Error parsing {:#?} as uint32 for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for program {} is not found", key, prog_name);
                None
            }
        }
    }
}

impl GetValByKey for u16 {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<u16> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_i64() {
                Some(b) => match u16::try_from(b) {
                    Ok(c) => Some(c),
                    Err(_) => {
                        eprintln!(
                            "Error parsing {:#?} as u16 for {} field in {}",
                            b, key, prog_name,
                        );
                        None
                    }
                },
                None => {
                    eprintln!(
                        "Error parsing {:#?} as u16 for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for program {} is not found", key, prog_name);
                None
            }
        }
    }
}

impl GetValByKey for PathBuf {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<PathBuf> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_str() {
                Some(b) => Some(PathBuf::from(b)),
                None => {
                    eprintln!(
                        "Error parsing {:#?} as path for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for programm {} is not found", key, prog_name);
                None
            }
        }
    }
}

impl GetValByKey for bool {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<bool> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_bool() {
                Some(b) => Some(b),
                None => {
                    eprintln!(
                        "Error parsing {:#?} as bool for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for programm {} is not found", key, prog_name);
                None
            }
        }
    }
}

impl GetValByKey for Vec<u32> {
    fn get_val_by_key(
        root: &linked_hash_map::LinkedHashMap<Yaml, Yaml>,
        key: &str,
        prog_name: &str,
    ) -> Option<Vec<u32>> {
        match root.get(&Yaml::String(String::from(key))) {
            Some(a) => match a.as_vec() {
                Some(b) => {
                    let mut resulting_vector = Vec::<u32>::with_capacity(2);
                    for code in b.iter() {
                        match code.as_i64() {
                            Some(c) => match u32::try_from(c) {
                                Ok(d) => resulting_vector.push(d),
                                Err(_) => {
                                    eprintln!(
                                        "Error parsing {} as u32 for {} in {}",
                                        c, key, prog_name
                                    );
                                }
                            },
                            None => {
                                eprintln!(
                                    "Error parsing {:#?} as i64 for {} in {}",
                                    b, key, prog_name
                                );
                            }
                        }
                    }
                    Some(resulting_vector)
                }
                None => {
                    eprintln!(
                        "Error parsing {:#?} as vec for {} field in {}",
                        a, key, prog_name
                    );
                    None
                }
            },
            None => {
                eprintln!("Field {} for programm {} is not found", key, prog_name);
                None
            }
        }
    }
}

pub fn get_working_dir_from_cmd(cmd: &str) -> PathBuf {
    match cmd.split_whitespace().next() {
        Some(a) => PathBuf::from(a),
        None => PathBuf::from("/"),
    }
}

pub fn create_yaml_structs(k: &Yaml, v: &Yaml) -> Option<Task> {
    let prog_name = match k.as_str() {
        Some(a) => a,
        None => {
            eprintln!("Invalid programm name {:#?}", k);
            return None;
        }
    };
    let programm_params = match v.as_hash() {
        Some(a) => a,
        None => {
            eprintln!("Error parsing body of {}", prog_name);
            return None;
        }
    };
    let cmd = match String::get_val_by_key(programm_params, "cmd", prog_name) {
        Some(a) => a,
        None => {
            return None;
        }
    };
    let numprocs = match u16::get_val_by_key(programm_params, "numprocs", prog_name) {
        Some(a) => a,
        None => {
            eprintln!("No numprocs for {} is given. Using default 1", prog_name);
            1u16
        }
    };
    let umask = match u16::get_val_by_key(programm_params, "umask", prog_name) {
        Some(a) => a,
        None => {
            eprintln!("Umask is not set. Using default 000");
            0
        }
    };
    let working_dir = match PathBuf::get_val_by_key(programm_params, "workingdir", prog_name) {
        Some(a) => a,
        None => {
            eprintln!(
                "Error parsing working dir for {}. Setting default.",
                prog_name
            );
            get_working_dir_from_cmd(&cmd)
        }
    };
    const BOOL_MESSAGE: &str = "Setting default value: false";

    let autostart = match bool::get_val_by_key(programm_params, "autostart", prog_name) {
        Some(a) => a,
        None => {
            eprintln!("{}", BOOL_MESSAGE);
            false
        }
    };

    let autorestart = match programm_params.get(&Yaml::String(String::from("autorestart"))) {
        Some(a) => match a.as_str() {
            Some(b) => match b.to_lowercase().as_str() {
                "unexpected" => false,
                "expected" => true,
                _ => {
                    eprintln!(
                        "Failed parsing autorestart for {}. Autorestart: {:#?}",
                        prog_name, b
                    );
                    false
                }
            },
            None => {
                eprintln!(
                    "Failed parsing autorestart for {}. Autorestart: {:#?}",
                    prog_name, a
                );
                false
            }
        },
        None => {
            eprintln!("Autorestart field for {} is not found.", prog_name);
            false
        }
    };
    let mut exitcodes = match programm_params.get(&Yaml::String(String::from("exitcodes"))) {
        Some(a) => match a.as_vec() {
            None => {
                eprintln!(
                    "Failed parsing exitcodes for {}. Exitcodes: {:#?}",
                    prog_name, a
                );
                vec![0]
            }
            Some(b) => {
                let mut resulting_vector = Vec::<u32>::with_capacity(2);
                for code in b.iter() {
                    resulting_vector.push(match code.as_i64() {
                        Some(c) => c as u32,
                        None => 0,
                    });
                }
                resulting_vector
            }
        },
        None => {
            eprintln!("Exitcodes for {} not found. Setting default.", prog_name);
            vec![0]
        }
    };
    println!(
        "PROGNAME: {} CMD: {} NUMPROCS: {} UMASK: {} WORKING_DIR: {}\n\
         AUTOSTART: {}, AUTORESTART: {}, EXITCODES: {:?}",
        prog_name,
        cmd,
        numprocs,
        umask,
        working_dir.display(),
        autostart,
        autorestart,
        exitcodes
    );
    None
}

pub fn read_config(config_path: &Path) {
    let file = File::open(config_path);
    let mut file = match file {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Error opening {}", config_path.display());
            std::process::exit(1);
        }
    };
    let mut file_data: String = String::new();
    file.read_to_string(&mut file_data).expect("Empty file");
    let d = YamlLoader::load_from_str(&file_data).expect("empty file");
    let document = &d[0].as_hash().expect("Unwrap of YAML failed");
    let root_element = document.get(&Yaml::String(String::from("programs")));
    let root_element = match root_element {
        Some(a) => a,
        None => {
            eprintln!("Root element not found.");
            std::process::exit(1);
        }
    };
    let root_map = root_element.as_hash().unwrap();
    for (k, v) in root_map {
        create_yaml_structs(k, v);
    }
}

//	println!("{:#?}", root_map);

//#[cfg(test)]
pub fn task_list_check() {
    let path = String::from("/home/odm3n/dev/taskmaster/server/src/config_reader/test_data.yaml");
    read_config(Path::new(&path));
}
