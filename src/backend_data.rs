use crate::stable_hash::StableHash;
use byteorder::{LittleEndian, ReadBytesExt};
use dashmap::DashMap;
use std::cmp::PartialEq;
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
pub struct FileData {
    pub r#type: String,
    pub namespace: String,
    pub sub_class: String,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug)]
pub struct MethodData {
    pub sub_class: String,
    pub name: String,
    pub requires_authority: bool,
    pub method_type: MethodType,
    pub parameters: DashMap<String, String>,
    pub rpcs: Vec<String>,
    pub sync_vars: DashMap<u8, FileData>,
}

#[derive(Debug)]
pub struct SyncVarData {
    pub sub_class: String,
    pub name: String,
    pub r#type: String,
    pub dirty_bit: u32,
    pub initial_value: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct BackendData {
    pub version: u16,
    pub components: DashMap<String, DashMap<u8, String>>,
    pub sync_var_datas: Vec<SyncVarData>,
    pub methods: Vec<MethodData>,
    pub assets: DashMap<u32, String>,
    pub scenes: DashMap<u64, String>,
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

fn read_components<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        //  key
        let key = read_string(reader);
        // value
        let len = reader.read_u16::<LittleEndian>().unwrap();
        let mut value = DashMap::new();
        for _ in 0..len {
            let k = reader.read_u8().unwrap();
            let v = read_string(reader);
            value.insert(k, v);
        }
        // insert
        data.components.insert(key, value);
    }
}

fn read_sync_vars<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        let sub_class = read_string(reader);
        let name = read_string(reader);
        let r#type = read_string(reader);
        let dirty_bit = reader.read_u32::<LittleEndian>().unwrap();
        let len = reader.read_u16::<LittleEndian>().unwrap();
        let mut initial_value = Vec::new();
        for _ in 0..len {
            initial_value.push(reader.read_u8().unwrap());
        }
        println!("initial_value:{:?}", initial_value);
        data.sync_var_datas.push(SyncVarData {
            sub_class,
            name,
            r#type,
            dirty_bit,
            initial_value,
        });
    }
}

fn read_file_data<T: Read>(reader: &mut T) -> FileData {
    FileData {
        r#type: read_string(reader),
        namespace: read_string(reader),
        sub_class: read_string(reader),
        name: read_string(reader),
        full_name: read_string(reader),
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
        let mut parameters = DashMap::new();
        for _ in 0..parameters_length {
            let key = read_string(reader);
            let value = read_string(reader);
            parameters.insert(key, value);
        }
        let mut rpcs = Vec::new();
        let mut sync_vars = DashMap::new();

        if method_type == MethodType::Command {
            let rpc_length = reader.read_u16::<LittleEndian>().unwrap();
            for _ in 0..rpc_length {
                rpcs.push(read_string(reader));
            }

            let sync_vars_length = reader.read_u16::<LittleEndian>().unwrap();
            for _ in 0..sync_vars_length {
                let key = reader.read_u8().unwrap();
                let value = read_file_data(reader);
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
    for _ in 0..length {
        let key = reader.read_u32::<LittleEndian>().unwrap();
        let value = read_string(reader);
        data.assets.insert(key, value);
    }
}

fn read_scenes<T: Read>(reader: &mut T, data: &mut BackendData) {
    let length = reader.read_u16::<LittleEndian>().unwrap();
    for _ in 0..length {
        let key = reader.read_u64::<LittleEndian>().unwrap();
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
        components: DashMap::new(),
        sync_var_datas: Vec::new(),
        methods: Vec::new(),
        assets: DashMap::new(),
        scenes: DashMap::new(),
    };

    while binary_reader.position() < binary_reader.get_ref().len() as u64 {
        let r#type = binary_reader.read_u16::<LittleEndian>().unwrap();
        match r#type {
            // 1 => read_components(&mut binary_reader, &mut data),
            2 => {
                println!("st read_sync_vars");
                read_sync_vars(&mut binary_reader, &mut data);
                println!("ed read_sync_vars");
            }
            3 => read_methods(&mut binary_reader, &mut data),
            4 => read_components(&mut binary_reader, &mut data),
            5 => read_assets(&mut binary_reader, &mut data),
            6 => read_scenes(&mut binary_reader, &mut data),
            _ => (),
        }
    }
    // println!("VERSION:{}\n{:?}", version, data);
    data
}
