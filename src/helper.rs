use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::bitcoincore_rpc_json::{Timestamp};
use bitcoincore_rpc::bitcoincore_rpc_json::{WalletProcessPsbtResult, WalletCreateFundedPsbtResult};
use bitcoin;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::Write;
use home::home_dir;
use secp256k1::{rand, Secp256k1, SecretKey};
use serde_json::{to_string};

//get the current user
pub fn get_user() -> String {
	home_dir().unwrap().to_str().unwrap().to_string().split("/").collect::<Vec<&str>>()[2].to_string()
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

//used to store a string param as a file
pub fn store_string(string: String, file_name: &String) -> Result<String, String> {
	let mut file_ref = match std::fs::File::create(file_name) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	file_ref.write_all(&string.as_bytes()).expect("Could not write string to file");
	Ok(format!("SUCCESS stored with no problems"))
}

//used to store a PSBT param as a file
pub fn store_psbt(psbt: &WalletProcessPsbtResult, file_name: String) -> Result<String, String> {
    let mut file_ref = match std::fs::File::create(file_name) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
    let psbt_json = to_string(&psbt).unwrap();
    file_ref.write_all(&psbt_json.to_string().as_bytes()).expect("Could not write string to file");
    Ok(format!("SUCCESS stored with no problems"))
 }

 //used to store a PSBT param as a file
pub fn store_unsigned_psbt(psbt: &WalletCreateFundedPsbtResult, file_name: String) -> Result<String, String> {
    let mut file_ref = match std::fs::File::create(file_name) {
        Ok(file) => file,
        Err(err) => return Err(err.to_string()),
    };
    let psbt_json = to_string(&psbt).unwrap();
    file_ref.write_all(&psbt_json.to_string().as_bytes()).expect("Could not write string to file");
    Ok(format!("SUCCESS stored with no problems"))
 }

//update the config.txt with the provided params
pub fn write(name: String, value:String) {
	let mut config_file = home_dir().expect("could not get home directory");
    config_file.push("config.txt");
    let mut written = false;
    let mut newfile = String::new();
    //read the config file to a string
    let contents = match fs::read_to_string(&config_file) {
        Ok(ct) => ct,
        Err(_) => {
            "".to_string()       
        }
    };
    //split the contents of the string
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
    //write the contents to the config file
    file.write_all(newfile.as_bytes()).expect("Could not rewrite file");
}

//used to check the mountpoint of /media/$USER/CDROM
pub fn check_cd_mount() -> std::string::String {
    //find the mountpoint of /media/user/CDROM
	let output = Command::new("df").args(["-h", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
	if !output.status.success() {
		let er = "error";
		return format!("{}", er)
	}
    //obtain the stdout of the result from above
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
			return format!("success")
		}else{
            continue
        }
	}
    //check if filepath exists
    let b = std::path::Path::new(&("/media/".to_string()+&get_user()+"/CDROM")).exists();
    //if CD mount path does not exist...create it and mount the CD
    if b == false{
        //create the dir
        let output = Command::new("sudo").args(["mkdir", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
            if !output.status.success() {
                return format!("error");
            }
        //mount the CDROM
        let output = Command::new("sudo").args(["mount", "/dev/sr0", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
        if !output.status.success() {
            return format!("error");
        }
    //if CD mount path already exists...mount the CD
    } else {
        //mount the CDROM
        let output = Command::new("sudo").args(["mount", "/dev/sr0", &("/media/".to_string()+&get_user()+"/CDROM")]).output().unwrap();
            if !output.status.success() {
                return format!("error");
            }
	}
	format!("success")
}

//generate an extended public and private keypair
pub fn generate_keypair() -> Result<(String, String), bitcoin::Error> {
	let secp = Secp256k1::new();
    let seed = SecretKey::new(&mut rand::thread_rng()).secret_bytes();
    let xpriv = bitcoin::bip32::ExtendedPrivKey::new_master(bitcoin::Network::Bitcoin, &seed).unwrap();
	let xpub = bitcoin::bip32::ExtendedPubKey::from_priv(&secp, &xpriv);
	Ok((bitcoin::base58::check_encode_slice(&xpriv.encode()), bitcoin::base58::check_encode_slice(&xpub.encode())))
}

//returns the checksum of the descriptor param
pub fn get_descriptor_checksum(descriptor: String) -> String {
    let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
		Ok(client)=> client,
		Err(err)=> return format!("{}", err.to_string())
	};
    //retrieve descriptor info
    let desc_info = match client.get_descriptor_info(&descriptor){
        Ok(info)=> info,
		Err(err)=> return format!("{}", err.to_string())
    };
    println!("Descriptor info: {:?}", desc_info);
    //parse the checksum
    let checksum = desc_info.checksum;
    println!("Checksum: {:?}", checksum);
    let output = &(descriptor.to_string() + "#" + &checksum.to_string());
    println!("output: {:?}", output);
    format!("{}", output)
}

//converts a unix timestamp to block height
pub fn unix_to_block_height(unix_timestamp: i64) -> i64 {
    let genesis_timestamp = 1231006505; //unix timestamp of genesis block
                            // 126230400 4 year period
    let block_interval = 600; //10 minutes in seconds
    let time_since_genesis = unix_timestamp - genesis_timestamp;
    let block_height = time_since_genesis / block_interval;
    block_height
}

//retrieve decay time from the file and output as Timestamp type
pub fn retrieve_decay_time(file: String) -> Timestamp {
	let decay_time_exists = std::path::Path::new(&("/mnt/ramdisk/sensitive/decay/".to_string()+&file.to_string())).exists();
	if decay_time_exists == true{
        //read the decay time file to a string
		let decay_time: String = match fs::read_to_string(&("/mnt/ramdisk/sensitive/decay/".to_string()+&file.to_string())){
			Ok(decay_time)=> decay_time,
			//return default timestamp
			Err(..)=> return Timestamp::Time(1676511266)
		};
        //parse the decay_time
		match decay_time.trim().parse() {
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

//retrieve start time from the decay_time file and output as integer
pub fn retrieve_decay_time_integer(file: String) -> i64 {
	let decay_time_exists = std::path::Path::new(&("/mnt/ramdisk/sensitive/decay/".to_string()+&file.to_string())).exists();
	if decay_time_exists == true{
        //read the decay_time file to a string
		let decay_time: String = match fs::read_to_string(&("/mnt/ramdisk/sensitive/decay/".to_string()+&file.to_string())){
			Ok(decay_time)=> decay_time,
			//return default time stamp
			Err(..)=> return 0
		};
        //parse the decay_time
		match decay_time.trim().parse() {
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

//retrieve the current unix time
pub fn retrieve_current_time_integer() -> i64{
    let current_time_res = Command::new("date").args(["+%s"]).output().unwrap();
    let current_time_str = std::str::from_utf8(&current_time_res.stdout).unwrap();
    match current_time_str.trim().parse() {
        Ok(result) => 
        return result,
        Err(..) => 
        //return default timestamp 
        return 0
}
}
