use ::migration::{SourceFile, TargetFile};
use std::io;
use std::io::Write;

pub fn create_mapping(mut inputs: &mut Vec<SourceFile>, targets: &mut Vec<TargetFile>) {
    // try to exact match non-audio files, these are usually small so not really worth trying something else on these
    for input in inputs.iter_mut().filter(|f| !f.is_audio) {
        match targets.iter().position(|target| {
            if input.size == target.size && input.extension == target.extension {
                true
            }else{
                false
            }
        }) {
            Some(pos) => input.mapping = Some(pos),
            None => {}
        }
    }

    // start with a map by filename
    map_by_filename(&mut inputs, &targets);
    loop {
        println!("Mapping:");
        print_mapping(&inputs,&targets);
        println!("Enter 'c' to continue, 's' to try filesize remap, 'f' to try filename remap or 'm' to manually adjust [c]");
        let mut reply = String::new();
        io::stdin().read_line(&mut reply).unwrap();
        match reply.trim() {
            "" | "c" => {
                break;
            },
            "s" => {
                map_by_size(&mut inputs, &targets);
            },
            "f" => {
                map_by_filename(&mut inputs, &targets);
            },
            "m" => {
                map_manual(&mut inputs, &targets);
            },
            _ => {
                println!("Unrecognized option.");
            }
        }
    }

    // assign the mapping to the targets as well
    for (i, input) in inputs.iter().enumerate() {
        if let Some(mapping) = input.mapping {
            targets[mapping].mapping = Some(i);
        }
    }
}

// output the mapping
fn print_mapping(inputs: &Vec<SourceFile>, targets: &Vec<TargetFile>) {
    // determine the maximume length for padding
    let max = inputs.iter().map(|e| e.display.len()).max().unwrap();
    // print all the mappings we found
    for input in inputs.iter() {
        if let Some(ref i) = input.mapping {
            println!("  {:2$} => {}", input.display, targets[*i].path.to_string_lossy(), max+1);
        } else {
            println!("  {:1$} => None", input.display, max+1);
        }
    }
}

// map by the filename
fn map_by_filename(inputs: &mut Vec<SourceFile>, targets: &Vec<TargetFile>) {
    let mut targets_audio:Vec<&TargetFile> = targets.iter().filter(|f| f.is_audio).collect();
    targets_audio.sort_by(|a, b| a.path.cmp(&b.path));
    for (i, input) in inputs.iter_mut().filter(|f| f.is_audio).enumerate() {
        if let Some(e) = targets_audio.get(i) {
            input.mapping = Some(e.index);
        } else {
            input.mapping = None
        }
    }
}

// map by filesize
fn map_by_size(inputs: &mut Vec<SourceFile>, targets: &Vec<TargetFile>) {
    let mut inputs_sorted:Vec<&mut SourceFile> = inputs.iter_mut().filter(|f| f.is_audio).collect();
    inputs_sorted.sort_by(|a, b| a.size.cmp(&b.size));
    let mut targets_sorted:Vec<&TargetFile> = targets.iter().filter(|f| f.is_audio).collect();
    targets_sorted.sort_by(|a, b| a.size.cmp(&b.size));
    for (i, input) in inputs_sorted.iter_mut().enumerate() {
        if let Some(e) = targets_sorted.get(i) {
            input.mapping = Some(e.index);
        } else {
            input.mapping = None
        }
    }
}

// manual adjust of the mapping
fn map_manual(inputs: &mut Vec<SourceFile>, targets: &Vec<TargetFile>) {
    println!("Listing target files and their index:");
    for item in targets.iter().filter(|f| f.is_audio) {
        println!(" {} | {}", item.index, item.path.to_string_lossy());
    }

    println!("Adjust the number file by file, enter to leave unchanged or 'n' to remove mapping:");
    // determine the maximum length for padding
    let max = inputs.iter().filter(|f| f.is_audio).map(|e| e.display.len()).max().unwrap();
    for item in inputs.iter_mut().filter(|f| f.is_audio) {
        let current = match item.mapping {
            Some(i) => {
                format!("{} | {}", targets[i].index, targets[i].path.to_string_lossy())
            },
            None => {
                "None".to_string()
            }
        };

        print!(" {:2$} [{}] ", item.display, current, max);
        io::stdout().flush().unwrap();
        let mut reply = String::new();
        io::stdin().read_line(&mut reply).unwrap();
        match reply.trim() {
            "n" => {
                item.mapping = None
            },
            _ => {
                let num = reply.trim().parse::<usize>(); //i32::from_str(&reply);
                if num.is_ok() {
                    let index = num.unwrap();
                    if index < targets.len() {
                        item.mapping = Some(index);
                    } else {
                        println!("Invalid index, ignoring input.");
                    }
                }
            }
        }
    }
}