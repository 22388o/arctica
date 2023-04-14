use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::{Auth, Client, Error, RawTx};
use bitcoincore_rpc::bitcoincore_rpc_json::{AddressType, ImportDescriptors, Timestamp};
use bitcoincore_rpc::bitcoincore_rpc_json::{GetRawTransactionResult, WalletProcessPsbtResult, CreateRawTransactionInput, ListTransactionResult, Bip125Replaceable, GetTransactionResultDetailCategory, WalletCreateFundedPsbtOptions, WalletCreateFundedPsbtResult, FinalizePsbtResult};
use bitcoin;
use bitcoin::locktime::Time;
use bitcoin::Address;
use bitcoin::consensus::serialize;
use bitcoin::consensus::deserialize;
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::util::amount::SignedAmount;
use bitcoin::Amount;
use bitcoin::Txid;
use bitcoin::Transaction;
use bitcoin::psbt::Psbt;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::collections::BTreeMap;
use home::home_dir;
use secp256k1::{rand, Secp256k1, SecretKey};
use secp256k1::rand::Rng;
use tauri::State;
use std::{thread, time::Duration};
use std::path::Path;
use std::process::Stdio;
use std::io::BufReader;
use std::any::type_name;
use std::num::ParseIntError;
use hex;
use serde_json::{json, to_string, Value};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::mem;
use base64::decode;

//get the current user
pub fn get_user() -> String {
	home_dir().unwrap().to_str().unwrap().to_string().split("/").collect::<Vec<&str>>()[2].to_string()
}

//only useful when running the application in a dev envrionment
//prints & error messages must be passed to the front end in a promise when running from a precompiled binary
pub fn print_rust(data: &str) -> String {
	println!("input = {}", data);
	format!("completed with no problems")
}

//determine the data type of the provided variable
pub fn type_of<T>(_: &T) -> &'static str{
	type_name::<T>()
}


//get the current $HOME path
pub fn get_home() -> String {
	home_dir().unwrap().to_str().unwrap().to_string()
}

//check for the presence of an internal storage uuid and if one is mounted, return it
pub fn get_uuid() -> String {
	//Obtain the internal storage device UUID if mounted
	let devices = Command::new(&("ls")).args([&("/media/".to_string()+&get_user())]).output().unwrap();
	if !devices.status.success() {
	return format!("ERROR in parsing /media/user");
	} 
	//convert the list of devices above into a vector of results
	let devices_output = std::str::from_utf8(&devices.stdout).unwrap();
	let split = devices_output.split('\n');
	let devices_vec: Vec<_> = split.collect();
	//loop through the vector and check the character count of each entry to obtain the uuid which is 36 characters
	let mut uuid = "none";
	for i in devices_vec{
		if i.chars().count() == 36{
			uuid = i.trim();
		} 
	}
	//if a valid uuid is not found, this function returns the string: "none"
	format!("{}", uuid)
}

//check if target path is empty
pub fn is_dir_empty(path: &str) -> bool {
	if let Ok(mut entries) = fs::read_dir(path){
		return entries.next().is_none();
	}
	false
}

//used to store keypairs & descriptors as a file
pub fn store_string(string: String, file_name: &String) -> Result<String, String> {
	let mut fileRef = match std::fs::File::create(file_name) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	fileRef.write_all(&string.as_bytes());
	Ok(format!("SUCCESS stored with no problems"))
}

//used to store the generated PSBT as a file
pub fn store_psbt(psbt: &WalletProcessPsbtResult, file_name: String) -> Result<String, String> {
    let mut fileRef = match std::fs::File::create(file_name) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
    let psbt_json = to_string(&psbt).unwrap();
    fileRef.write_all(&psbt_json.to_string().as_bytes());
    Ok(format!("SUCCESS stored with no problems"))
 }

//copy any shards potentially on the recovery CD to ramdisk
pub fn copy_shards_to_ramdisk() {
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard1.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard2.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard3.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard4.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard5.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard6.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard7.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard8.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard9.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard10.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
	Command::new("cp").args([&("/media/".to_string()+&get_user()+"/CDROM/shards/shard11.txt"), "/mnt/ramdisk/shards"]).output().unwrap();
}

//update the config.txt with the provided params
pub fn write(name: String, value:String) {
	let mut config_file = home_dir().expect("could not get home directory");
    config_file.push("config.txt");
    let mut written = false;
    let mut newfile = String::new();

    let contents = match fs::read_to_string(&config_file) {
        Ok(ct) => ct,
        Err(_) => {
            "".to_string()       
        }
    };

    for line in contents.split("\n") {
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() == 2 {
           let (n,v) = (parts[0],parts[1]); 
           newfile += n;
           newfile += "=";
           if n == name {
            newfile += &value;
            written = true;
           } else {
            newfile += v;
           }
           newfile += "\n";
        }
    }

    if !written {
        newfile += &name;
        newfile += "=";
        newfile += &value;
    }

    let mut file = File::create(&config_file).expect("Could not Open file");
    file.write_all(newfile.as_bytes()).expect("Could not rewrite file");
}


//used to check the mountpoint of /media/$USER/CDROM
pub fn check_cd_mount() -> std::string::String {
	let mut mounted = "false";
	let output = Command::new("df").args(["-h", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
	if !output.status.success() {
		let er = "error";
		return format!("{}", er)
	}
		
	let df_output = std::str::from_utf8(&output.stdout).unwrap();
	//use a closure to split the output of df -h /media/$USER/CDROM by whitespace and \n
	let split = df_output.split(|c| c == ' ' || c == '\n');
	let output_vec: Vec<_> = split.collect();
	//loop through the vector
	for i in output_vec{
		println!("new line:");
		println!("{}", i);
		//if any of the lines contain /dev/sr0 we know that /media/$USER/CDROM is mounted correctly
		if i == "/dev/sr0"{
			mounted = "true";
			return format!("success")
		}
	}
	if mounted == "false"{
		//check if filepath exists
		let b = std::path::Path::new(&("/media/".to_string()+&get_user()+"/CDROM")).exists();
		//if CD mount path does not exist...create it and mount the CD
		if b == false{
			let output = Command::new("sudo").args(["mkdir", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
				if !output.status.success() {
					return format!("error");
				}
			let output = Command::new("sudo").args(["mount", "/dev/sr0", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
			if !output.status.success() {
				return format!("error");
			}
		//if CD mount path already exists...mount the CD
		} else {
			let output = Command::new("sudo").args(["mount", "/dev/sr0", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
				if !output.status.success() {
					return format!("error");
				}
		}
	}
	format!("success")
}

//used to generate an extended public and private keypair
pub fn generate_keypair() -> Result<(String, String), bitcoin::Error> {
	let secp = Secp256k1::new();
    let seed = SecretKey::new(&mut rand::thread_rng()).secret_bytes();
    let xpriv = bitcoin::util::bip32::ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed).unwrap();
	let xpub = bitcoin::util::bip32::ExtendedPubKey::from_priv(&secp, &xpriv);
	Ok((bitcoin::util::base58::check_encode_slice(&xpriv.encode()), bitcoin::util::base58::check_encode_slice(&xpub.encode())))
}

//builds the high security descriptor, 7 of 11 thresh with decay. 4 of the 11 keys will go to the BPS
pub fn build_high_descriptor(keys: &Vec<String>, sdcard: &String) -> Result<String, String> {
	println!("calculating 4 year block time span");
    let start_time = retrieve_start_time_integer(); 
	println!("start time: {}", start_time);
	let start_time_block_height = unix_to_block_height(start_time);
	println!("start time block height: {}", start_time_block_height);
	//add the 4 year time delay in seconds 12623400
	let four_years_unix_time = 126230400 + start_time;
	let four_years_block_height = unix_to_block_height(four_years_unix_time);
	println!("for years block height: {}", four_years_block_height);
	let four_years = start_time_block_height + four_years_block_height;
	println!("four years: {}", four_years);
	//establish 1 month in estimated block height change
    let month = 4383;
	println!("reading xpriv");
	let xpriv = fs::read_to_string(&("/mnt/ramdisk/sensitive/private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
	println!("{}", xpriv);
	if sdcard == "1"{
		println!("Found sdcard = 1");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output: String = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "2"{
		println!("Found sdcard = 2");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "3"{
		println!("Found sdcard = 3");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "4"{
		println!("Found sdcard = 4");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "5"{
		println!("Found sdcard = 5");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "6"{
		println!("Found sdcard = 6");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], xpriv, keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "7"{
		println!("Found sdcard = 7");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], xpriv, four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "timemachine1"{
		println!("Found sdcard = timemachine1");
		let timemachinexpriv = fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), timemachinexpriv, keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if sdcard == "timemachine2"{
		println!("Found sdcard = timemachine2");
		let timemachinexpriv = fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], timemachinexpriv, keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if sdcard == "timemachine3"{
		println!("Found sdcard = timemachine3");
		let timemachinexpriv = fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], timemachinexpriv, keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if sdcard == "timemachine4"{
		println!("Found sdcard = timemachine4");
		let timemachinexpriv = fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], timemachinexpriv, four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else{
		println!("no valid sdcard param found, creating read only desc");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("Read only DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}

}

//builds the medium security descriptor, 2 of 7 thresh with decay. 
pub fn build_med_descriptor(keys: &Vec<String>, sdcard: &String) -> Result<String, String> {
	println!("calculating 4 year block time span");
    let start_time = retrieve_start_time_integer(); 
	println!("start time: {}", start_time);
	let start_time_block_height = unix_to_block_height(start_time);
	println!("start time block height: {}", start_time_block_height);
	//add the 4 year time delay in seconds 12623400
	let four_years_unix_time = 126230400 + start_time;
	let four_years_block_height = unix_to_block_height(four_years_unix_time);
	println!("for years block height: {}", four_years_block_height);
	let four_years = start_time_block_height + four_years_block_height;
	println!("four years: {}", four_years);
	//establish 1 month in estimated block height change
    let month = 4383;
	println!("reading xpriv");
	let xpriv = fs::read_to_string(&("/mnt/ramdisk/sensitive/private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
	println!("{}", xpriv);
	if sdcard == "1"{
		println!("Found sdcard = 1");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "2"{
		println!("Found sdcard = 2");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "3"{
		println!("Found sdcard = 3");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "4"{
		println!("Found sdcard = 4");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "5"{
		println!("Found sdcard = 5");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "6"{
		println!("Found sdcard = 6");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], xpriv, keys[3], keys[4], xpriv, keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if sdcard == "7"{
		println!("Found sdcard = 7");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], xpriv, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else{
		println!("no valid sdcard param found, creating read only desc");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}
}

//builds the low security descriptor, 1 of 7 thresh, used for tripwire
//TODO this needs to use its own special keypair or it will be a privacy leak once implemented
//TODO this may not need child key designators /* because it seems to use hardened keys but have not tested this descriptor yet
	pub fn build_low_descriptor(keys: &Vec<String>, sdcard: &String) -> Result<String, String> {
		println!("reading xpriv");
		let xpriv = fs::read_to_string(&("/mnt/ramdisk/sensitive/private_key".to_string()+&(sdcard.to_string()))).expect(&("Error reading public_key from file".to_string()+&(sdcard.to_string())));
		println!("{}", xpriv);
		if sdcard == "1"{
			println!("Found sdcard = 1");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "2"{
			println!("Found sdcard = 2");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "3"{
			println!("Found sdcard = 3");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "4"{
			println!("Found sdcard = 4");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "5"{
			println!("Found sdcard = 5");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "6"{
			println!("Found sdcard = 6");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], keys[4], xpriv, keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if sdcard == "7"{
			println!("Found sdcard = 7");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], xpriv);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else{
			println!("No valid sd card param found, creating read only desc");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}
	
	}

//returns the checksum of the descriptor param
pub fn get_descriptor_checksum(descriptor: String) -> String {
    let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
    let desc_info = Client.get_descriptor_info(&descriptor).unwrap();
    println!("Descriptor info: {:?}", desc_info);
    let checksum = desc_info.checksum;
    println!("Checksum: {:?}", checksum);
    let output = &(descriptor.to_string() + "#" + &checksum.to_string());
    println!("output: {:?}", output);
    format!("{}", output)
}


//converts a unix timestamp to block height
pub fn unix_to_block_height(unix_timestamp: i64) -> i64 {
    let genesis_timestamp = 1231006505; //unix timestamp of genesis block
                            // 1671299369 start time
                            // 126230400 4 year period
    let block_interval = 600; //10 minutes in seconds
    let time_since_genesis = unix_timestamp - genesis_timestamp;
    let block_height = time_since_genesis / block_interval;
    block_height
}



//RPC command
// ./bitcoin-cli createwallet "wallet name" true true
//creates a blank watch only walket
pub fn create_wallet(wallet: String, sdcard: &String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
	let output = match Client.create_wallet(&(wallet.to_string()+"_wallet"+&sdcard.to_string()), None, Some(true), None, None) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	Ok(format!("SUCCESS creating the wallet {:?}", output))
}

//RPC command
// ./bitcoin-cli -rpcwallet=<filepath>|"wallet_name" importdescriptors "requests"
//requests is a JSON and is formatted as follows
//'[{"desc": "<descriptor goes here>", "active":true, "range":[0,100], "next_index":0, "timestamp": <start_time_timestamp>}]'
//acceptable params here are "low", "immediate", "delayed"
//TODO timestamp is not currently fucntional due to a type mismatch, timestamp within the ImportDescriptors struct wants bitcoin::timelock:time
pub fn import_descriptor(wallet: String, sdcard: &String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+ &(sdcard.to_string())), auth).expect("could not connect to bitcoin core");
	let desc: String = fs::read_to_string(&("/mnt/ramdisk/sensitive/descriptors/".to_string()+&(wallet.to_string())+"_descriptor" + &(sdcard.to_string()))).expect("Error reading reading descriptor from file");
	let start_time = retrieve_start_time();
	let output = match Client.import_descriptors(ImportDescriptors {
		descriptor: desc,
		timestamp: start_time,
		active: Some(true),
		range: Some((0, 100)),
		next_index: Some(0),
		internal: Some(true),
		label: None
	}){
			Ok(file) => file,
			Err(err) => return Err(err.to_string()),
		
	};
	Ok(format!("Success in importing descriptor...{:?}", output))
}

//retrieve start time from the start_time file and output as Timestamp type
pub fn retrieve_start_time() -> Timestamp {
	let start_time_complete = std::path::Path::new(&(get_home()+"/start_time")).exists();
	if start_time_complete == true{
		let start_time: String = fs::read_to_string(&(get_home()+"/start_time")).expect("could not read start_time");
		let result = match start_time.trim().parse() {
			Ok(result) => 
			return Timestamp::Time(result),
			Err(..) => 
			//return default timestamp 
			return Timestamp::Time(1676511266)
		};
	} else {
		//return default timestamp
		return Timestamp::Time(1676511266)
	}
}

//retrieve start time from the start_time file and output as integer
pub fn retrieve_start_time_integer() -> i64 {
	let start_time_complete = std::path::Path::new(&(get_home()+"/start_time")).exists();
	if start_time_complete == true{
		let start_time: String = fs::read_to_string(&(get_home()+"/start_time")).expect("could not read start_time");
		let result = match start_time.trim().parse() {
			Ok(result) => 
			return result,
			Err(..) => 
			//return default timestamp 
			return 0
		};
	} else {
		//return default timestamp
		return 0
	}
}