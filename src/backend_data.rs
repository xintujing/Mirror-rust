use crate::stable_hash::StableHash;
use byteorder::{LittleEndian, ReadBytesExt};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum MethodType {
    Command = 1,
    TargetRpc = 2,
    ClientRpc = 3,
}
impl MethodType {
    fn from_u8(value: u8) -> Option<MethodType> {
        match value {
            1 => Some(MethodType::Command),
            2 => Some(MethodType::TargetRpc),
            3 => Some(MethodType::ClientRpc),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct MethodData {
    pub sub_class: String,
    pub name: String,
    pub requires_authority: bool,
    pub method_type: MethodType,
    pub parameters: HashMap<String, String>,
    pub rpcs: Vec<String>,
    pub sync_vars: HashMap<u8, String>,
}

#[derive(Debug)]
pub struct SyncVarData {
    pub sub_class: String,
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Default)]
pub struct BackendData {
    pub version: u16,
    pub dirty_bits: HashMap<String, u32>,
    pub sync_vars: Vec<SyncVarData>,
    pub methods: Vec<MethodData>,
    pub assets: HashMap<u32, String>,
    pub scenes: HashMap<u64, String>,
}

impl BackendData {
    pub fn get_method(&self, hash_code: u16) -> Option<&MethodData> {
        for method_data in &self.methods {
            if method_data.name.get_fn_stable_hash_code() == hash_code {
                return Some(method_data);
            }
        }
        None
    }
}

fn read_string<T: Read>(reader: &mut T) -> String {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    let mut buffer = vec![0; length as usize];
    reader.read_exact(&mut buffer).unwrap();
    String::from_utf8_lossy(&buffer).to_string()
}

fn read_dirty_bits<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        let key = read_string(reader);
        let value = reader.read_u32::<LittleEndian>().unwrap();
        data.dirty_bits.insert(key, value);
    }
}

fn read_sync_vars<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        let sub_class = read_string(reader);
        let name = read_string(reader);
        let type_ = read_string(reader);
        data.sync_vars.push(SyncVarData {
            sub_class,
            name,
            type_,
        });
    }
}

fn read_methods<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        let sub_class = read_string(reader);
        let name = read_string(reader);
        let requires_authority = reader.read_u8().unwrap() != 0;
        let method_type = MethodType::from_u8(reader.read_u8().unwrap()).unwrap();
        let parameters_length = reader.read_u16::<LittleEndian>().unwrap();
        let mut parameters = HashMap::new();
        for _ in 0..parameters_length {
            let key = read_string(reader);
            let value = read_string(reader);
            parameters.insert(key, value);
        }
        let mut rpcs = Vec::new();
        let mut sync_vars = HashMap::new();

        if method_type == MethodType::Command {
            let rpc_length = reader.read_u16::<LittleEndian>().unwrap();
            for _ in 0..rpc_length {
                rpcs.push(read_string(reader));
            }

            let sync_vars_length = reader.read_u16::<LittleEndian>().unwrap();
            for _ in 0..sync_vars_length {
                let key = reader.read_u8().unwrap();
                let value = read_string(reader);
                sync_vars.insert(key, value);
            }
        }
        data.methods.push(MethodData {
            sub_class,
            name,
            requires_authority,
            method_type,
            parameters,
            rpcs,
            sync_vars,
        });
    }
}

fn read_assets<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for i in 0..length {
        let key = reader.read_u32::<LittleEndian>().unwrap();
        let value = read_string(reader);
        println!("read_assets {} {} {}", i, key, value);
        data.assets.insert(key, value);
    }
}

fn read_scenes<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    println!("read_scenes {}", length);
    for i in 0..length {
        let key = reader.read_u64::<LittleEndian>().unwrap();
        println!("read_scenes {} {}", i, key);
        let value = read_string(reader);
        data.scenes.insert(key, value);
    }
}


pub fn import() -> BackendData {
    let file_path = "_backend_data.bin";
    let mut file = File::open(file_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut binary_reader = std::io::Cursor::new(buffer);

    let version = binary_reader.read_u16::<LittleEndian>().unwrap();

    let mut data = BackendData {
        version,
        dirty_bits: HashMap::new(),
        sync_vars: Vec::new(),
        methods: Vec::new(),
        assets: HashMap::new(),
        scenes: HashMap::new(),
    };

    while binary_reader.position() < binary_reader.get_ref().len() as u64 {
        let type_ = binary_reader.read_u16::<LittleEndian>().unwrap();
        match type_ {
            1 => read_dirty_bits(&mut binary_reader, &mut data),
            2 => read_sync_vars(&mut binary_reader, &mut data),
            3 => read_methods(&mut binary_reader, &mut data),
            4 => read_assets(&mut binary_reader, &mut data),
            5 => read_scenes(&mut binary_reader, &mut data),
            _ => (),
        }
    }
    println!("VERSION:{}\n{:?}", version, data);

    data
}
