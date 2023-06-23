//WARNING: Never use snake_case for function params that will be invoked by tauri, it converts them to camelCase and breaks the app

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use bitcoincore_rpc::{Client};
use std::sync::{Mutex};
use std::process::Command;
use std::fs;
use std::fs::File;
use home::home_dir;

//import functions from helper.rs
mod helper;
use helper::{
	get_user, get_home, is_dir_empty, get_uuid,
	write, check_cd_mount, retrieve_decay_time_integer
};

//import functions from setup.rs
mod setup;
use setup::{
	init_iso, create_bootable_usb, create_setup_cd, generate_store_key_pair, 
	generate_store_simulated_time_machine_key_pair, create_descriptor, install_hw_deps, distribute_shards_hw2, 
	distribute_shards_hw3, distribute_shards_hw4, distribute_shards_hw5, distribute_shards_hw6,
	distribute_shards_hw7, create_backup, make_backup,
};

//import functions from bitcoin.rs
mod bitcoin_wallet;
use bitcoin_wallet::{
	get_address, get_balance, get_transactions, generate_psbt, start_bitcoind, start_bitcoind_network_off,
	stop_bitcoind, decode_processed_psbt, broadcast_tx, finalize_psbt, sign_processed_psbt, export_psbt, get_blockchain_info, 
	load_wallet, get_descriptor_info, decode_funded_psbt, sign_funded_psbt, retrieve_median_blocktime
};

// std::env::set_var("RUST_LOG", "bitcoincore_rpc=debug");

struct TauriState(Mutex<Option<Client>>);

#[tauri::command]
//for testing only
async fn test_function() -> String {
	format!("testing")
}

#[tauri::command]
//current the config currently in $HOME
//conditional logic that determines application state is set by the front end after reading is completed
fn read() -> std::string::String {
    let mut config_file = home_dir().expect("could not get home directory");
    println!("{}", config_file.display());
    config_file.push("config.txt");
	//read the config file in $HOME to string
    let contents = match fs::read_to_string(&config_file) {
        Ok(ct) => ct,
        Err(_) => {
        	"".to_string()
        }
    };
	//split the config string
    for line in contents.split("\n") {
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() == 2 {
            let (n,v) = (parts[0],parts[1]);
            println!("read line: {}={}", n, v);
        }
    }
    format!("{}", contents)
}

//copy the contents of the currently inserted CD to the ramdisk /mnt/ramdisk/CDROM
#[tauri::command]
async fn copy_cd_to_ramdisk() -> String {
	Command::new("sleep").args(["4"]).output().unwrap();
	//check if a CDROM is inserted
	let a = std::path::Path::new("/dev/sr0").exists();
	if a == false {
		let er = "ERROR in copy_cd_to_ramdisk: No CD inserted";
		return format!("{}", er)
	}
	//check if CDROM is mounted at the proper filepath, if not, mount it
	let mounted = check_cd_mount().to_string();
	if mounted == "error" {
		let er = "ERROR in copy_cd_to_ramdisk: error checking CD mount";
		return format!("{}", er)
	}
	//copy cd contents to ramdisk
	let output = Command::new("cp").args(["-R", &("/media/".to_string()+&get_user()+"/CDROM"), "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in copying CD contents = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	//open up permissions
	let output = Command::new("sudo").args(["chmod", "-R", "777", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in opening file permissions of CDROM = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS in coyping CD contents")
}

//read the config file of the currently inserted CD/DVD
#[tauri::command]
fn read_cd() -> std::string::String {
	Command::new("sleep").args(["4"]).output().unwrap();
	//check if a CDROM is inserted
	let a = std::path::Path::new("/dev/sr0").exists();
	if a == false {
		let er = "ERROR in read_CD: No CD inserted";
		return format!("{}", er)
	}
	//check if CDROM is mounted at the proper filepath, if not, mount it
	let mounted = check_cd_mount();
	if mounted == "error" {
		let er = "ERROR in read_CD: error checking CD mount";
		return format!("{}", er)
	}
	//check for config
    // let config_file = "/mnt/ramdisk/CDROM/config.txt";
	let config_file = &("/media/".to_string()+&get_user()+"/CDROM/"+"config.txt");
    let contents = match fs::read_to_string(&config_file) {
        Ok(ct) => ct,
        Err(_) => {
        	"".to_string()
        }
    };
	//split the config string
    for line in contents.split("\n") {
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() == 2 {
            let (n,v) = (parts[0],parts[1]);
            println!("read line: {}={}", n, v);
        }
    }
    format!("{}", contents)
}

#[tauri::command]
//blank and rewrite the currently inserted disc with the contents of /mnt/ramdisk/CDROM
async fn refresh_cd() -> String {
	//create iso from CD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR refreshing CD with genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//check if the CDROM is blank
	let dir_path = "/media/ubuntu/CDROM";
	let is_empty = is_dir_empty(dir_path);
	//unmount the disc
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	//if not blank, wipe the CD
	if is_empty == false{
		let output = Command::new("sudo").args(["wodim", "-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR refreshing setupCD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	//burn setupCD iso to the setupCD
	let output = Command::new("sudo").args(["wodim", "dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing CD with burning iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing CD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in refreshing CD")
}

//eject the current disc
#[tauri::command]
async fn eject_cd() -> String {
	//copy cd contents to ramdisk
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	format!("SUCCESS in ejecting CD")
}

//pack up and encrypt the contents of the sensitive directory in ramdisk into an encrypted directory on the current Hardware Wallet
#[tauri::command]
async fn packup() -> String {
	println!("packing up sensitive info");
	//remove stale encrypted dir
	Command::new("sudo").args(["rm", &(get_home()+"/encrypted.gpg")]).output().unwrap();
	//remove stale tarball
	Command::new("sudo").args(["rm", "/mnt/ramdisk/unecrypted.tar"]).output().unwrap();
	//pack the sensitive directory into a tarball
	let output = Command::new("tar").args(["cvf", "/mnt/ramdisk/unencrypted.tar", "/mnt/ramdisk/sensitive"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in packup = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	//encrypt the sensitive directory tarball 
	let output = Command::new("gpg").args(["--batch", "--passphrase-file", "/mnt/ramdisk/CDROM/masterkey", "--output", &(get_home()+"/encrypted.gpg"), "--symmetric", "/mnt/ramdisk/unencrypted.tar"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in packup = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	format!("SUCCESS in packup")
}

//decrypt & unpack the contents of an encrypted directory on the current Hardware Wallet into the sensitive directory in ramdisk
#[tauri::command]
async fn unpack() -> String {
	println!("unpacking sensitive info");
	//remove stale tarball(We don't care if it fails/succeeds)
	Command::new("sudo").args(["rm", "/mnt/ramdisk/decrypted.out"]).output().unwrap();
	//decrypt sensitive directory
	let output = Command::new("gpg").args(["--batch", "--passphrase-file", "/mnt/ramdisk/CDROM/masterkey", "--output", "/mnt/ramdisk/decrypted.out", "-d", &(get_home()+"/encrypted.gpg")]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	// unpack sensitive directory tarball
	let output = Command::new("tar").args(["xvf", "/mnt/ramdisk/decrypted.out", "-C", "/mnt/ramdisk/"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
    // copy sensitive dir to ramdisk
	let output = Command::new("cp").args(["-R", "/mnt/ramdisk/mnt/ramdisk/sensitive", "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	// remove nested sensitive tarball output
	Command::new("sudo").args(["rm", "-r", "/mnt/ramdisk/mnt"]).output().unwrap();
	// #NOTES:
	// #can use this to append files to a decrypted tarball without having to create an entire new one
	// #tar rvf output_tarball ~/filestobeappended
	format!("SUCCESS in unpack")
}

//create and mount the ramdisk directory for holding senstive data at /mnt/ramdisk
#[tauri::command]
async fn create_ramdisk() -> String {
	//check if the ramdisk already exists and has been used by Arctica this session
	let a = std::path::Path::new("/mnt/ramdisk/sensitive").exists();
	let b = std::path::Path::new("/mnt/ramdisk/CDROM").exists();
    if a == true || b == true{
		return format!("Ramdisk already exists");
	}
	else{
		//ramdisk is empty but the filepath exists
		let c = std::path::Path::new("/mnt/ramdisk").exists();
		if c == true{
			let output = Command::new("sudo").args(["rm", "-r", "/mnt/ramdisk"]).output().unwrap();
			if !output.status.success() {
				return format!("Error in removing stale /mnt/ramdisk")
			}
		}
		//disable swapiness
		let output = Command::new("sudo").args(["swapoff", "-a"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in disabling swapiness {}", std::str::from_utf8(&output.stderr).unwrap());
			}
		//create the ramdisk
		let output = Command::new("sudo").args(["mkdir", "/mnt/ramdisk"]).output().unwrap();
		if !output.status.success() {
		return format!("ERROR in making /mnt/ramdisk dir {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//allocate the RAM for ramdisk 
		let output = Command::new("sudo").args(["mount", "-t", "ramfs", "-o", "size=250M", "ramfs", "/mnt/ramdisk"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in Creating Ramdisk = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//open ramdisk file permissions
		let output = Command::new("sudo").args(["chmod", "777", "/mnt/ramdisk"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in Creating Ramdisk = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//make the target dir for encrypted payload to or from Hardware Wallets
		let output = Command::new("mkdir").args(["/mnt/ramdisk/sensitive"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in Creating /mnt/ramdiskamdisk/sensitive = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//make the debug.log file
		let output = Command::new("echo").args(["/mnt/ramdisk/debug.log"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in Creating debug.log = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	format!("SUCCESS in Creating Ramdisk")
	}
}

#[tauri::command]
//mount the internal storage drive at /media/$USER/$UUID
//and symlinks internal .bitcoin/chainstate and ./bitcoin/blocks
//the below internal drive configurations assume a default ubuntu install on the internal disk without any custom partitioning
async fn mount_internal() -> String {
	//Obtain the internal storage device UUID if already mounted
	let mut uuid = get_uuid();
	//mount internal drive if nvme
	if uuid == "ERROR in parsing /media/user" {
		return format!("Error in parsing /media/user to get uuid")
	}
	else if uuid == "none"{
		//mount the internal drive if NVME
		Command::new("udisksctl").args(["mount", "--block-device", "/dev/nvme0n1p2"]).output().unwrap();
		//mount internal drive if SATA
		Command::new("udisksctl").args(["mount", "--block-device", "/dev/sda2"]).output().unwrap();
		//Attempt to shut down bitcoin core
		let output = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli")).args(["stop"]).output().unwrap();
		//bitcoin core shutdown fails (meaning it was not running)...
		if output.status.success() {
			//function succeeds, core is running for some reason, wait 15s for daemon to stop
			Command::new("sleep").args(["15"]).output().unwrap();
		}
		//obtain the UUID of the currently mounted internal storage drive
		uuid = get_uuid();
		//error in get_uuid()
		if uuid == "ERROR in parsing /media/user" {
			return format!("Error in parsing /media/user to get uuid")
		}
		//no uuid found
		else if uuid == "none" {
			return format!("ERROR could not find a valid UUID in /media/$user");
		}
		//obtain the username of the internal storage device
		let host = Command::new(&("ls")).args([&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home")]).output().unwrap();
		if !host.status.success() {
			return format!("ERROR in parsing /media/user/uuid/home {}", std::str::from_utf8(&host.stderr).unwrap());
		} 
		let host_user = std::str::from_utf8(&host.stdout).unwrap().trim();
		//open the file permissions for local host user dir
		let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string()))]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in opening internal storage dir file permissions {}", std::str::from_utf8(&output.stderr).unwrap());
		} 
		//make internal storage bitcoin dotfiles at /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin/blocks & /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin/chainstate
		let c = std::path::Path::new(&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin/blocks")).exists();
		let d = std::path::Path::new(&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin/chainstate")).exists();
		if c == false && d == false{
			let output = Command::new("sudo").args(["mkdir", "--parents", &("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin/blocks"), &("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin/chainstate") ]).output().unwrap();
			if !output.status.success() {
			return format!("ERROR in removing stale ./bitcoin/chainstate dir {}", std::str::from_utf8(&output.stderr).unwrap());
			}
		}
		//open file permissions of internal storage dotfile dirs
		let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin")]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in opening file permissions of internal storage .bitcoin dirs {}", std::str::from_utf8(&output.stderr).unwrap());
		} 
		let e = std::path::Path::new(&("/media/ubuntu/".to_string()+&(uuid.to_string()))).exists();
		if e == true{
			format!("SUCCESS in mounting the internal drive")
		}else{
			format!("ERROR mounting internal drive, final check failed")
		}
	}//in the following condition, get_uuid() returns a valid uuid.
	// So we can assume that the internal drive is already mounted
	else {
		format!("SUCCESS internal drive is already mounted")
	}
}

//calculate time until next decay
#[tauri::command]
async fn calculate_decay_time(file: String) -> String {
	//retrieve start time
	let current_time_str = retrieve_median_blocktime();
	let current_time: i64 = current_time_str.parse().unwrap();
	//retrieve immediate_decay
	let decay_time = retrieve_decay_time_integer(file.to_string());
	//subtract start_time from immediate decay
	let time = decay_time - current_time;
	//convert to years, months, days, hours, minutes
	let years = time / 31536000; //divide by number of seconds in a year
	let mut remainder = time % 31536000;
	let months = remainder / 2592000; //divide by number of seconds in a month
	remainder = remainder % 2592000;
	let weeks = remainder / 604800; //divide by number of seconds in a week
	remainder = remainder % 604800;
	let days = remainder / 86400; //divide by number of seconds in a day
	remainder = remainder % 86400;
	let hours = remainder / 3600; //divide by number of seconds in an hour
	remainder = remainder % 3600;
	let minutes = remainder / 60;
	remainder = remainder % 60;
	//  day
	if years <= 0 && months <= 0 && weeks <= 0 && days <= 0 && hours <= 0 && minutes <= 0 {
		format!("decay complete")
	}
	else{
		format!("years={}, months={}, weeks={}, days={}, hours={}, minutes={}, seconds={}", years, months, weeks, days, hours, minutes, remainder)
	}
}

//used to combine recovered shards into an encryption/decryption masterkey
#[tauri::command]
async fn combine_shards() -> String {
	println!("combining shards in /mnt/ramdisk/shards");
	//execute the combine-shards bash script
	let output = Command::new("bash")
		.args([get_home()+"/scripts/combine-shards.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

//for updating config values from the front end
#[tauri::command]
async fn async_write(name: &str, value: &str) -> Result<String, String> {
    write(name.to_string(), value.to_string());
    println!("{}", name);
    Ok(format!("completed with no problems"))
}

//check the currently inserted CD for an encryption masterkey
#[tauri::command]
async fn check_for_masterkey() -> String {
	println!("checking ramdisk for masterkey");
    let b = std::path::Path::new("/mnt/ramdisk/CDROM/masterkey").exists();
    if b == true{
        format!("masterkey found")
    }
	else{
        format!("key not found")
    }
}

#[tauri::command]
//this fn is used to store decryption shards gathered from various Hardware Wallets to eventually be reconstituted into a masterkey when attempting to log in manually
async fn recovery_initiate() -> String {
	//create the CDROM dir if it does not already exist
	let a = std::path::Path::new("/mnt/ramdisk/CDROM").exists();
	if a == false{
		let output = Command::new("mkdir").args(["/mnt/ramdisk/CDROM"]).output().unwrap();
		if !output.status.success() {
		return format!("ERROR in creating recovery CD, with making CDROM dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	}
	//create recoveryCD config, this informs the front end on BOOT whether or not the user is attempting to manually recover login or attempting to sign a PSBT
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=recoverycd" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in creating recovery CD, with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//collect shards from Hardware Wallets for export to transfer CD
	let output = Command::new("cp").args(["-R", &(get_home()+"/shards"), "/mnt/ramdisk/CDROM/shards"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in creating recovery CD with copying shards from HW = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	//create iso from transferCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR creating recovery CD with creating ISO = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//wipe the CD 
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("sudo").args(["wodim", "-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn transferCD iso to the transfer CD
	Command::new("sudo").args(["wodim", "dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	let output = Command::new("sudo").args(["wodim", "-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in creating recovery CD")
}

//calculate the number of encryption shards currently in the ramdisk
#[tauri::command]
async fn calculate_number_of_shards() -> u32 {
	let mut x = 0;
    for _file in fs::read_dir("/mnt/ramdisk/CDROM/shards").unwrap() {
		x = x + 1;
	}
	return x;
}

#[tauri::command]
async fn collect_shards() -> String {
	println!("collecting shards");
	//obtain a list of all of the filenames in $HOME/shards
	let shards = Command::new(&("ls")).args([&(get_home()+"/shards")]).output().unwrap();
	if !shards.status.success() {
	return format!("ERROR in collect_shards() with parsing $HOME/shards");
	} 
	//convert the list of shards into a vector of results
	let shards_output = std::str::from_utf8(&shards.stdout).unwrap();
	let split = shards_output.split('\n');
	let shards_vec: Vec<_> = split.collect();
	//iterate through the vector and copy each file to /mnt/ramdisk/CDROM/shards
	for i in shards_vec{
		let output = Command::new("cp").args([&(get_home()+"/shards"+&(i.to_string())), "/mnt/ramdisk/CDROM/shards"]).output().unwrap();
		if !output.status.success() {
			return format!("Error in collect_shards() with copying shards")
		}
		} 
	format!("SUCCESS in collecting shards")
}

#[tauri::command]
//convert the completed recovery CD to a Transfer CD via config file
async fn convert_to_transfer_cd() -> String {
	//remove stale config
	let output = Command::new("sudo").args(["rm", "/mnt/ramdisk/CDROM/config.txt"]).output().unwrap();
	if !output.status.success() {
		return format!("Error in convert to transfer CD with removing stale config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//create transferCD config
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=transfercd" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in converting to transfer CD, with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in converting config to transfer CD")
}

// #[tauri::command]
// //for testing only
// async fn init_test() -> String {
//     let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
//     let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
	// 	Ok(client)=> client,
	// 	Err(err)=> return Ok(format!("{}", err.to_string()))
	// };
//     let mut keys = Vec::new();
//     let (mut xpriv, mut xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     (xpriv, xpub) = generate_keypair().expect("could not gen keypair");
//     keys.push(xpub);
//     let desc = build_high_descriptor(&client, &keys).unwrap();
//     format!("testing {} {}", desc, desc.sanity_check().unwrap() == ())
// }

#[tauri::command]
async fn display_qr() -> String{
	let output = Command::new("eog").args(["--disable-gallery", "--new-instance", "/mnt/ramdisk/qrcode.png"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in displaying QR code with EOG = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("successfully displayed QR code")
}

fn main() {
  	tauri::Builder::default()
	//export all tauri functions to be handled by the front end
  	.manage(TauriState(Mutex::new(None))) 
  	.invoke_handler(tauri::generate_handler![
        //init_test,
        test_function,
        create_bootable_usb,
        create_setup_cd,
        read_cd,
        copy_cd_to_ramdisk,
		eject_cd,
        init_iso,
        async_write,
        read,
        combine_shards,
        mount_internal,
        create_ramdisk,
        packup,
        unpack,
        install_hw_deps,
        refresh_cd,
		calculate_decay_time,
        distribute_shards_hw2,
        distribute_shards_hw3,
        distribute_shards_hw4,
        distribute_shards_hw5,
        distribute_shards_hw6,
        distribute_shards_hw7,
    	create_descriptor,
        create_backup,
        make_backup,
        start_bitcoind,
        start_bitcoind_network_off,
		stop_bitcoind,
        check_for_masterkey,
        recovery_initiate,
        calculate_number_of_shards,
        collect_shards,
        convert_to_transfer_cd,
		generate_store_key_pair,
		generate_store_simulated_time_machine_key_pair,
		load_wallet,
		get_address,
		get_balance,
	    get_transactions,
		get_descriptor_info,
		get_blockchain_info,
		generate_psbt,
		export_psbt,
		sign_processed_psbt,
		sign_funded_psbt,
		finalize_psbt,
		broadcast_tx,
		decode_processed_psbt,
		decode_funded_psbt,
		display_qr,
		retrieve_median_blocktime,
        ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}