extern crate failure;
extern crate regex;

use failure::Error;
use regex::Regex;
use std::io::{Read, BufReader};
use std::path::PathBuf;

struct MachineName {
    name: String
}

struct StorageControllerName {
    name: String
}

struct UUID {
    uuid: String
}

fn get_args() -> (MachineName, StorageControllerName) {
    let mut args = std::env::args().skip(1);
    let storagectl_name = StorageControllerName {
        name: args.next().unwrap_or("SATA Controller".to_string())
    };
    let machine_name = MachineName {
        name: args.next().unwrap_or("default".to_string())
    };
    (machine_name, storagectl_name)
}

fn get_uuid<'a>(machine_name: &'a MachineName) -> Result<UUID, Error> {
    let mut id_path = PathBuf::from(r".vagrant\machines");
    id_path.push(&machine_name.name);
    id_path.push("virtualbox");
    id_path.push("id");

    if !id_path.exists() {
        return Err(failure::err_msg(format!("The id file is not found: {:?}", id_path)));
    }

    let mut reader = BufReader::new(std::fs::File::open(&id_path)?);

    let mut uuid = String::new();
    let _ = reader.read_to_string(&mut uuid)?;

    Ok(UUID { uuid })
}

fn has_sata_controller<'a>(uuid: &'a UUID, storagectl_name: &'a StorageControllerName) -> Result<bool, Error> {
    let vboxmanage_path = PathBuf::from(r"C:\Program Files\Oracle\VirtualBox\VBoxManage.exe");
    if !vboxmanage_path.exists() {
        return Err(failure::err_msg(format!("VBoxManage.exe is not found: {:?}", vboxmanage_path)));
    }

    let output = std::process::Command::new(vboxmanage_path)
        .args(&["showvminfo", &uuid.uuid])
        .output()?;

    let re = Regex::new(r"^Storage Controller Name \(\d+\):\s*(.+)")?;
    let has_sata_controller = String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| {
            re.captures(&line)
                .and_then(|caps| Some(caps.get(1)?.as_str() == storagectl_name.name))
                .unwrap_or(false)
        });

    Ok(has_sata_controller)
}

fn main() -> Result<(), Error> {
    let (machine_name, storagectl_name) = get_args();
    let uuid = get_uuid(&machine_name)?;
    let has_sata_controller = has_sata_controller(&uuid, &storagectl_name)?;

    println!("{}", if has_sata_controller { 1 } else { 0 });

    Ok(())
}
