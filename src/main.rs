use winapi::ctypes::c_void;
use std::thread::sleep;
use std::time::Duration;

use toy_arms::external::{read, write};
use toy_arms::VirtualKeyCode;
use toy_arms::external::Process;

use simulate;
use simulate::Key;

#[derive(Debug)]
struct TextObject {
    text_list: Vec<String>,
    cur_idx: u32,
    magic_type: u32,
}

fn read_multilevel_pointer<T>(process_handle: *mut c_void, base_addr: usize, offsets: &Vec<usize>) -> Option<T> {
    if offsets.len() == 0 {
        return None;
    }
    let mut cur_address = base_addr;
    for i in 0..(offsets.len() - 1) {
        cur_address += offsets[i];
        let output = read::<usize>(process_handle, cur_address);
        if output.is_err() {
            return None;
        }
        cur_address = output.unwrap();
    }
    cur_address += offsets.last().unwrap();
    let output = read::<T>(process_handle, cur_address);
    if output.is_ok() {
        return Some(output.unwrap());
    }
    return None;
}

fn get_text_objects(process_handle: *mut c_void, typeing_manager_ptr: usize) -> Vec<TextObject> {
    let mut objects = Vec::new();
    let slot_list = read_multilevel_pointer::<usize>(process_handle, typeing_manager_ptr, &vec![0x88, 0x18]);
    if slot_list.is_none() || slot_list.unwrap() == 0 {
        return objects;
    }
    let slot_list = slot_list.unwrap();
    // 0x34 is the last index of the hashtable, but that works better, then the count
    let object_count = read_multilevel_pointer::<u32>(process_handle, typeing_manager_ptr, &vec![0x88, 0x34]);
    if object_count.is_none() || object_count.unwrap() == 0 {
        return objects;
    }
    let object_count = object_count.unwrap();
    for i in 0..object_count {
        let object_ptr = read_multilevel_pointer::<usize>(process_handle, slot_list, &vec![0x28 + 0x10 * i as usize, 0x18, 0x20]);
        if object_ptr.is_none() || object_ptr.unwrap() == 0 {
            continue;
        }
        let object_ptr = object_ptr.unwrap();
        let magic_ty = read_multilevel_pointer::<u32>(process_handle, object_ptr, &vec![0xE0]);
        if magic_ty.is_none() {
            continue;
        }
        let magic_ty = magic_ty.unwrap();
        let word_index = read_multilevel_pointer::<u32>(process_handle, object_ptr, &vec![0x104]);
        if word_index.is_none() {
            continue;
        }
        let word_index = word_index.unwrap();
        let word_array_size = read_multilevel_pointer::<u32>(process_handle, object_ptr, &vec![0xB8, 0x18]);
        if word_array_size.is_none() || word_array_size.unwrap() <= word_index {
            continue;
        }
        let word_array_size = word_array_size.unwrap();
        let mut obj: TextObject = TextObject { text_list: Vec::new(), cur_idx: word_index, magic_type: magic_ty };

        let word_items_ptr = read_multilevel_pointer::<usize>(process_handle, object_ptr, &vec![0xB8, 0x10]);
        if word_items_ptr.is_none() || word_items_ptr.unwrap() == 0 {
            continue;
        }
        let word_items_ptr = word_items_ptr.unwrap();
        for j in 0..word_array_size {
            let str_len = read_multilevel_pointer::<u32>(process_handle, word_items_ptr, &vec![0x20 + 0x20 * j as usize, 0x10]);
            if str_len.is_none() || str_len.unwrap() == 0 {
                continue;
            }
            let mut cur_text = String::new();
            let str_len = str_len.unwrap();
            for k in 0..str_len {
                let character = read_multilevel_pointer::<u16>(process_handle, word_items_ptr, &vec![0x20 + 0x20 * j as usize, 0x14 + 2 * k as usize]);
                if character.is_none() || character.unwrap() > 0xFF {
                    continue;
                }
                let character = character.unwrap();
                cur_text.push(char::from_u32(character as u32).unwrap());
            }
            obj.text_list.push(cur_text);
        }
        objects.push(obj);
    }


    return objects;
}


fn set_active_magic_type(magic_type: u32, ) -> u32 {
    if magic_type == 1 {
        let output = simulate::type_str("fire");
        if output.is_ok() {
            let _ = output.unwrap();
        }
    }
    else if magic_type == 2 {
        let output = simulate::type_str("ice");
        if output.is_ok() {
            let _ = output.unwrap();
        }
    }
    else if magic_type == 3 {
        let output = simulate::type_str("spark");
        if output.is_ok() {
            let _ = output.unwrap();
        }
    }
    else if magic_type == 4 {
        let output = simulate::type_str("wind");
        if output.is_ok() {
            let _ = output.unwrap();
        }
    }
    return magic_type;
}


fn main() {
    let mut process = Process::from_process_name("Epistory.exe");
    while !process.is_ok() {
        process = Process::from_process_name("Epistory.exe");
        sleep(Duration::from_secs(1));
    }
    let process = process.unwrap();
    let module_info = process.get_module_info("mono-2.0-bdwgc.dll").unwrap(); 

    let mut typing_manager_ptr: usize = 0;
    while typing_manager_ptr == 0{
        let output = read_multilevel_pointer::<usize>(process.process_handle, module_info.module_base_address, &vec![0x07191D8, 0x28, 0xA0, 0x28, 0x20]);
        if output.is_some() {
            typing_manager_ptr = output.unwrap() + 0x180;
        }
        else {
            sleep(Duration::from_secs(1));
        }
    }

    println!("TypingManagerPtr {:#16x}", typing_manager_ptr);
    let mut cur_active_element = 1;
    loop {
        let text_objects = get_text_objects(process.process_handle, typing_manager_ptr);
        if text_objects.is_empty() {
            sleep(Duration::from_millis(10));
        }
        for text_obj in text_objects {

            if text_obj.magic_type != cur_active_element && text_obj.magic_type != 0 {
                cur_active_element = set_active_magic_type(text_obj.magic_type);
            }
            for i in (text_obj.cur_idx as usize)..text_obj.text_list.len() {

                for c in text_obj.text_list[i].chars() {
                    let output = simulate::send(c);
                    if output.is_ok() {
                        let _ = output.unwrap();
                    }
                }
            }
        }

    }
}

