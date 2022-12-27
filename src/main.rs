#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::Auth;
use bitcoin;
use bdk::{Wallet};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoincore_rpc::Client;
use bdk::blockchain::ConfigurableBlockchain;
use bdk::blockchain::rpc::RpcBlockchain;
use bdk::blockchain::rpc::RpcConfig;
use bdk::blockchain::Blockchain;
use bdk::blockchain::GetHeight;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use home::home_dir;
use secp256k1::{rand, Secp256k1, SecretKey};
use tauri::State;
use std::{thread, time::Duration};
use std::path::Path;



struct TauriState(Mutex<RpcConfig>, Mutex<String>, Mutex<String>, Mutex<String>);

//helper function
fn print_rust(data: &str) -> String {
	println!("input = {}", data);
	format!("completed with no problems")
}

//helper function
fn get_user() -> String {
	home_dir().unwrap().to_str().unwrap().to_string().split("/").collect::<Vec<&str>>()[2].to_string()
}
fn get_home() -> String {
	home_dir().unwrap().to_str().unwrap().to_string()
}

//helper function
//copy any shards potentially on the recovery CD to ramdisk
fn copy_shards_to_ramdisk() {
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

fn write(name: String, value:String) {
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



#[tauri::command]
fn read() -> std::string::String {
    let mut config_file = home_dir().expect("could not get home directory");
    println!("{}", config_file.display());
    config_file.push("config.txt");
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
            println!("read line: {}={}", n, v);
        }
    }
    format!("{}", contents)
}

fn generate_private_key() -> Result<bitcoin::PrivateKey, bitcoin::Error> {
	let secp = Secp256k1::new();
	let secret_key = SecretKey::new(&mut rand::thread_rng());
	Ok(bitcoin::PrivateKey::new(secret_key, bitcoin::Network::Bitcoin))
}

fn derive_public_key(private_key: bitcoin::PrivateKey) -> Result<bitcoin::PublicKey, bitcoin::Error>  {
	let secp = Secp256k1::new();
	let secret_key = SecretKey::new(&mut rand::thread_rng());
	Ok(bitcoin::PublicKey::from_private_key(&secp, &private_key))
}

fn store_private_key(private_key: bitcoin::PrivateKey, file_name: String) -> Result<String, String> {
	let mut fileRef = match std::fs::File::create(file_name) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	fileRef.write_all(&private_key.to_bytes());
	Ok(format!("SUCCESS stored with no problems"))
}

fn store_public_key(public_key: bitcoin::PublicKey, file_name: String) -> Result<String, String> {
	let mut fileRef = match std::fs::File::create(file_name) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	fileRef.write_all(&public_key.to_bytes());
	Ok(format!("SUCCESS stored with no problems"))
}

#[tauri::command]
async fn generate_store_key_pair(number: String) -> String {
	//number corresponds to currentSD here and is provided by the front end
	let private_key_file = "/mnt/ramdisk/sensitive/private_key".to_string()+&number;
	let public_key_file = "/mnt/ramdisk/sensitive/public_key".to_string()+&number;
	let private_key = match generate_private_key() {
		Ok(private_key) => private_key,
		Err(err) => return "ERROR could not generate private_key: ".to_string()+&err.to_string()
	};
	let public_key = match derive_public_key(private_key) {
		Ok(public_key) => public_key,
		Err(err) => return "ERROR could not dervie public key: ".to_string()+&err.to_string()
	};
	match store_private_key(private_key, private_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store private key: ".to_string()+&err
	}
	match store_public_key(public_key, public_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store public key: ".to_string()+&err
	}

	//make the pubkey dir in the setupCD staging area, can fail or succeed
	Command::new("mkdir").args(["--parents", "/mnt/ramdisk/CDROM/pubkeys"]).output().unwrap();

	//copy public key to setupCD dir
	let output = Command::new("cp").args([&("/mnt/ramdisk/sensitive/public_key".to_string()+&number), "/mnt/ramdisk/CDROM/pubkeys"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in generate store key pair with copying pubkey= {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS generated and stored Private and Public Key Pair")
}

fn recover_private_key(file_name: String) -> Result<bitcoin::PrivateKey, String> {
	let private_key_string = match fs::read_to_string(file_name) {
		Ok(data) => data,
		Err(err) => return Err(err.to_string())
	};
	let private_key = match bitcoin::PrivateKey::from_slice(private_key_string.as_bytes(), bitcoin::Network::Bitcoin) {
		Ok(private_key) => private_key,
		Err(err) => return Err(err.to_string())
	};
	Ok(private_key)
}

fn recover_public_key(file_name: String) -> Result<bitcoin::PublicKey, String> {
	let public_key_string = match fs::read_to_string(file_name) {
		Ok(data) => data,
		Err(err) => return Err(err.to_string())
	};
	let public_key = match bitcoin::PublicKey::from_slice(public_key_string.as_bytes()) {
		Ok(public_key) => public_key,
		Err(err) => return Err(err.to_string())
	};
	Ok(public_key)
}

#[tauri::command]
async fn recover_key_pair() -> String {
	let private_key_file = "/mnt/ramdisk/sensitive/private_key.txt".to_string();
	let public_key_file = "/mnt/ramdisk/sensitive/private_key.txt".to_string();
	let private_key = match recover_private_key(private_key_file) {
		Ok(private_key) => private_key,
		Err(err) => return "ERROR could not recover private key: ".to_string()+&err
	};
	let public_key = match recover_public_key(public_key_file) {
		Ok(public_key) => public_key,
		Err(err) => return "ERROR could not recover public key: ".to_string()+&err
	};
	// Use These
	format!("SUCCESS recovered Private/Public Key Pair")
}

// fn build_high_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
// 	let mut keys = Vec::new();
// 	let ctx = Secp256k1::new();
// 	for i in 0..11 {
// 		keys.push(generate_key().expect("could not get key").public_key(&ctx));
// 		println!("test = {}", generate_key().expect("could not get key").public_key(&ctx));
// 	}
// 	let four_years = blockchain.get_height().unwrap()+210379;
// 	let month = 4382;
// 	let desc = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}),sun:after({}),sun:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({}),sun:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[7], keys[8], keys[9], keys[10], four_years, four_years);
// 	println!("DESC: {}", desc);
// 	Ok(miniscript::Descriptor::<bitcoin::PublicKey>::from_str(&desc).unwrap().to_string())
// }

// fn build_med_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
// 	let mut keys = Vec::new();
// 	let ctx = Secp256k1::new();
// 	for i in 0..7 {
// 		keys.push(generate_key().expect("could not get key").public_key(&ctx))
// 	}
// 	let four_years = blockchain.get_height().unwrap()+210379;
// 	let desc = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),sun:after({})))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years);
// 	Ok(miniscript::Descriptor::<bitcoin::PublicKey>::from_str(&desc).unwrap().to_string())
// }


// fn build_low_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
// 	let mut keys = Vec::new();
// 	let ctx = Secp256k1::new();
// 	for i in 0..7 {
// 		keys.push(generate_key().expect("could not get key").public_key(&ctx))
// 	}
// 	let desc = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6]);
// 	Ok(miniscript::Descriptor::<bitcoin::PublicKey>::from_str(&desc).unwrap().to_string())
// }

// #[tauri::command]
// fn generate_wallet(state: State<TauriState>) -> String {
// 	let blockchain = RpcBlockchain::from_config(&*state.0.lock().unwrap()).expect("failed to connect to bitcoin core(Ensure bitcoin core is running before calling this function)");
// 	*state.1.lock().unwrap() = build_high_descriptor(&blockchain).expect("failed to bulid high lvl descriptor");
// 	*state.2.lock().unwrap() = build_med_descriptor(&blockchain).expect("failed to bulid med lvl descriptor");
// 	*state.3.lock().unwrap() = build_low_descriptor(&blockchain).expect("failed to bulid low lvl descriptor");
// 	return "Completed With No Problems".to_string()
// }

// #[tauri::command]
// fn get_address_high_wallet(state: State<TauriState>) -> String {
// 	println!("test ");
// 	let desc: String = (*state.1.lock().unwrap()).clone();
// 	println!("desc = {}", desc);
// 	let wallet: Wallet<MemoryDatabase> = Wallet::new(&desc, None, bitcoin::Network::Bitcoin, MemoryDatabase::default()).expect("failed to bulid high lvl wallet");
// 	return wallet.get_address(bdk::wallet::AddressIndex::New).expect("could not get address").to_string()
// }


#[tauri::command]
async fn test_function() -> String {
	let file = File::create("/home/".to_string()+&get_user()+"/testfile.txt").unwrap();
	let output = Command::new("echo").args(["file contents go here" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in test function = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in test function")
}


// file paths for this script and create_bootable_usb will need to change for prod
//these paths assume the user is compiling the application with cargo run inside ~/arctica
#[tauri::command]
async fn init_iso() -> String {
	println!("obtaining & creating modified ubuntu iso");

	println!("removing stale writable");
	//remove writable if exists, developer failsafe
	Command::new("sudo").args(["rm", "-r", "-f", &("/media/".to_string()+&get_user()+"/writable")]).output().unwrap();

	println!("downloading kvm dependencies");
	//download KVM deps
	Command::new("sudo").args(["apt-get", "-y", "install", "qemu-system-x86", "qemu-kvm", "libvirt-clients", "libvirt-daemon-system", "bridge-utils"]).output().unwrap();
	
	//mkusb deps, deprecated as create_bootable no longer uses mkusb
	// sudo add-apt-repository -y universe
	// sudo add-apt-repository -y ppa:mkusb/ppa
	// sudo apt -y update
	// sudo apt install -y mkusb
	// sudo apt install -y usb-pack-efi

	//download dependencies required on each SD card
	Command::new("sudo").args(["apt", "update"]).output().unwrap();
	Command::new("sudo").args(["apt", "download", "wodim", "genisoimage", "ssss"]).output().unwrap();

	//check if ubuntu iso & bitcoin core already exists, and if no, obtain
	//NOTE: this currently checks the arctica repo but this will change once refactor is finished and user can run binary on host machine 
	println!("obtaining ubuntu iso and bitcoin core if needed");
	let a = std::path::Path::new("./ubuntu-22.04.1-desktop-amd64.iso").exists();
	let b = std::path::Path::new("./bitcoin-23.0-x86_64-linux-gnu.tar.gz").exists();
	if a == false{
		let output = Command::new("wget").args(["-O", "ubuntu-22.04.1-desktop-amd64.iso", "http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in init iso with downloading ubuntu iso = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	if b == false{
		let output = Command::new("wget").args(["https://bitcoincore.org/bin/bitcoin-core-23.0/bitcoin-23.0-x86_64-linux-gnu.tar.gz"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in init iso with downloading bitcoin core = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}

	println!("removing stale persistent isos");
	//remove stale persistent isos
	Command::new("sudo").args(["rm", "persistent-ubuntu.iso"]).output().unwrap();
	Command::new("sudo").args(["rm", "persistent-ubuntu1.iso"]).output().unwrap();
	println!("removing stale pid");
	//remove stale pid file
	Command::new("sudo").args(["rm", "pid.txt"]).output().unwrap();

	println!("modifying ubuntu iso to have persistence");
	//modify ubuntu iso to have persistence
	let output = Command::new("bash").args([&(get_home()+"/arctica/scripts/sed1.sh")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in running sed1 {}", std::str::from_utf8(&output.stderr).unwrap());
	} 
	let exists = Path::new(&(get_home()+"/arctica/persistent-ubuntu1.iso")).exists();
	if !exists {
		return format!("ERROR in running sed1, script completed but did not create iso");
	}

	println!("modifying ubuntu iso timeout");
	let output = Command::new("bash").args([&(get_home()+"/arctica/scripts/sed2.sh")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in running sed2 {}", std::str::from_utf8(&output.stderr).unwrap());
	} 
	let exists = Path::new(&(get_home()+"/arctica/persistent-ubuntu.iso")).exists();
	if !exists {
		return format!("ERROR in running sed2, script completed but did not create iso");
	}

	println!("removing stale persistent iso");
	//remove stale persistent iso
	Command::new("sudo").args(["rm", "persistent-ubuntu1.iso"]).output().unwrap();

	println!("fallocate persistent iso");
	//fallocate persistent iso
	let output = Command::new("fallocate").args(["-l", "5GiB", "persistent-ubuntu.iso"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with fallocate persistent iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("booting iso with kvm");
	//boot kvm to establish persistence
	let output = Command::new("kvm").args(["-m", "2048", &("/home/".to_string()+&get_user()+"/arctica/persistent-ubuntu.iso"), "-daemonize", "-pidfile", "pid.txt", "-cpu", "host", "-display", "none"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with kvm = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("sleeping for 200 seconds");
	// sleep for 200 seconds
	Command::new("sleep").args(["200"]).output().unwrap();

	println!("obtaining pid");
	//obtain pid
	let file = "./pid.txt";
	let pid = match fs::read_to_string(file){
		Ok(data) => data.replace("\n", ""),
		Err(err) => return format!("{}", err.to_string())
	};
	
	println!("killing pid");
	//kill pid
	let output = Command::new("kill").args(["-9", &pid]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with killing pid = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("mount persistent iso");
	//mount persistent iso at /media/$USER/writable/upper/
	let output = Command::new("udisksctl").args(["loop-setup", "-f", &("/home/".to_string()+&get_user()+"/arctica/persistent-ubuntu.iso")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with mounting persistent iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("sleep for 2 seconds");
	// sleep for 2 seconds
	Command::new("sleep").args(["2"]).output().unwrap();

	println!("opening file permissions for persistent dir");
	//open file permissions for persistent directory
	let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with opening file permissions of persistent dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("Making dependencies directory")
	//make dependencies directory
	Command::new("mkdir").args([&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")])

	println!("Copying dependencies")
	//copying over dependencies genisoimage
	let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/arctica/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//copying over dependencies ssss
	let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/arctica/ssss_0.5-5_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying ssss = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//copying over dependencies wodim
	let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/arctica/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying wodim = {}", std::str::from_utf8(&output.stderr).unwrap());
	}


	println!("copying arctica binary");
	//copy over artica binary and make executable
	let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/arctica/target/debug/app"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/arctica")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying arctica binary = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	println!("copying arctica icon");
	let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/arctica/icons/arctica.jpeg"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/arctica.jpeg")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying binary jpeg = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	println!("making arctica a .desktop file");
	let output = Command::new("sudo").args(["cp", &("/home/".to_string()+&get_user()+"/arctica/shortcut/Arctica.desktop"), &("/media/".to_string()+&get_user()+"/writable/upper/usr/share/applications/Arctica.desktop")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying arctica.desktop = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	
	println!("making arctica binary an executable");
	let output = Command::new("sudo").args(["chmod", "+x", &("/media/".to_string()+&get_user()+"/writable/upper/usr/share/applications/Arctica.desktop")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with making binary executable = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("copying scripts library");
	//copy over scripts library. 
	let output = Command::new("cp").args(["-r", &("/home/".to_string()+&get_user()+"/arctica/scripts"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with copying scripts dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("extracting bitcoin core");
	//extract bitcoin core
	let output = Command::new("tar").args(["-xzf", &("/home/".to_string()+&get_user()+"/arctica/bitcoin-23.0-x86_64-linux-gnu.tar.gz"), "-C", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with extracting bitcoin core = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("making local internal bitcoin dotfile");
	//make local internal bitcoin dotfile
	let output = Command::new("mkdir").args(["--parents", &("/home/".to_string()+&get_user()+"/.bitcoin/blocks"), &("/home/".to_string()+&get_user()+"/.bitcoin/chainstate")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with making local .bitcoin dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("create target device .bitcoin dir");
	//create target device .bitcoin dir
	let output = Command::new("mkdir").args([&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.bitcoin")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso with making target .bitcoin dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("create bitcoin.conf on target device");
	//create bitcoin.conf on target device
	let file = File::create(&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.bitcoin/bitcoin.conf")).unwrap();
	let output = Command::new("echo").args(["-e", "rpcuser=rpcuser\nrpcpassword=477028"]).stdout(file).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in init iso, with creating bitcoin.conf = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in init_iso")
}

//initial flash of all 7 SD cards
#[tauri::command]
async fn create_bootable_usb(number: String, setup: String) -> String {
    write("type".to_string(), "sdcard".to_string());
    write("sdNumber".to_string(), number.to_string());
    write("setupStep".to_string(), setup.to_string());
	println!("creating bootable ubuntu device = {} {}", number, setup);
	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//remove old config from iso
	Command::new("sudo").args(["rm", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/config.txt")]).output().unwrap();
	//copy new config
	let output = Command::new("sudo").args(["cp", &("/home/".to_string()+&get_user()+"/config.txt"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in creating bootable with copying current config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//open file permissions for config
	let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/config.txt")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in creating bootable with opening file permissions = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//remove current working config from local
	let output = Command::new("sudo").args(["rm", &("/home/".to_string()+&get_user()+"/config.txt")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in creating bootable with removing current working config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn iso with mkusb
	let output = Command::new("sudo").args(["dd", "bs=16M", &("if=/home/".to_string()+&get_user()+"/arctica/persistent-ubuntu.iso"), "of=/dev/sda", "status=progress", "oflag=direct"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in creating bootable with dd = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in creating bootable device")
}

#[tauri::command]
async fn create_setup_cd() -> String {
	println!("creating setup CD");
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	//install sd dependencies for wodim and ssss
	// let output = Command::new("sudo").args(["add-apt-repository", "-y", "universe"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }
	// let output = Command::new("sudo").args(["apt", "update"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }
	// //download wodim
	// let output = Command::new("sudo").args(["apt", "install", "-y", "wodim"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }
	// //download shamir secret sharing library
	// let output = Command::new("sudo").args(["apt", "install", "ssss"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }

	//install sd dependencies for genisoimage
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing genisoimage for create_setup_cd {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for ssss
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/ssss_0.5-5_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing ssss for create_setup_cd {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for wodim
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing wodim for create_setup_cd {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//create setupCD config
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=setupcd" ]).stdout(file).output().unwrap();

	//create masterkey and derive shards
	let output = Command::new("bash").args([&(get_home()+"/scripts/create-setup-cd.sh")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in running create-setup-cd.sh {}", std::str::from_utf8(&output.stderr).unwrap());
	} 
	//NOTE: EVENTUALLY THE APPROPRIATE SHARDS NEED TO GO TO THE BPS HERE

	//copy first 2 shards to SD 1
	let output = Command::new("sudo").args(["cp", "/mnt/ramdisk/shards/shard1.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in copying shard1.txt in create setup CD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	let output = Command::new("sudo").args(["cp", "/mnt/ramdisk/shards/shard11.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in copying shard11.txt in create setup CD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//remove stale shard file
	let output = Command::new("sudo").args(["rm", "/mnt/ramdisk/shards_untrimmed.txt"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in removing deprecated shards_untrimmed in create setup cd = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//stage setup CD dir with shards for distribution
	let output = Command::new("sudo").args(["cp", "-R", "/mnt/ramdisk/shards", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in copying shards to CDROM dir in create setup cd = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//create iso from setupCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/setupCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR refreshing setupCD with genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR refreshing setupCD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//burn setupCD iso to the setupCD
	let output = Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/setupCD.iso"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with burning iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in Creating Setup CD")

}

#[tauri::command]
async fn copy_cd_to_ramdisk() -> String {
	//copy cd contents to ramdisk
	let output = Command::new("cp").args(["-R", &("/media/".to_string()+&get_user()+"/CDROM"), "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in copying CD contents = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//open up permissions
	let output = Command::new("sudo").args(["chmod", "-R", "777", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in opening file permissions of CDROM = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS in coyping CD contents")
}

#[tauri::command]
async fn packup() -> String {
	println!("packing up sensitive info");
	//remove stale encrypted dir
	Command::new("sudo").args(["rm", &("/home/".to_string()+&get_user()+"/encrypted.gpg")]).output().unwrap();

	//remove stale tarball
	Command::new("sudo").args(["rm", "/mnt/ramdisk/unecrypted.tar"]).output().unwrap();

	//pack the sensitive directory into a tarball
	let output = Command::new("tar").args(["cvf", "/mnt/ramdisk/unencrypted.tar", "/mnt/ramdisk/sensitive"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in packup = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//encrypt the sensitive directory tarball 
	let output = Command::new("gpg").args(["--batch", "--passphrase-file", "/mnt/ramdisk/CDROM/masterkey", "--output", &("/home/".to_string()+&get_user()+"/encrypted.gpg"), "--symmetric", "/mnt/ramdisk/unencrypted.tar"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in packup = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS in packup")

}

#[tauri::command]
async fn unpack() -> String {
	println!("unpacking sensitive info");

	//remove stale tarball(We don't care if it fails/succeeds)
	Command::new("sudo").args(["rm", "/mnt/ramdisk/decrypted.out"]).output().unwrap();


	//decrypt sensitive directory
	let output = Command::new("gpg").args(["--batch", "--passphrase-file", "/mnt/ramdisk/CDROM/masterkey", "--output", "/mnt/ramdisk/decrypted.out", "-d", &("/home/".to_string()+&get_user()+"/encrypted.gpg")]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	// unpack sensitive directory tarball
	let output = Command::new("tar").args(["xvf", "/mnt/ramdisk/decrypted.out", "-C", "/mnt/ramdisk/"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

    // copy sensitive dir to ramdisk
	let output = Command::new("cp").args(["-R", "/mnt/ramdisk/mnt/ramdisk/sensitive", "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in unpack = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	// remove nested sensitive
	Command::new("sudo").args(["rm", "-r", "/mnt/ramdisk/mnt"]).output().unwrap();

	// #NOTES:
	// #use this to append files to a decrypted tarball without having to create an entire new one
	// #tar rvf output_tarball ~/filestobeappended
	format!("SUCCESS in unpack")
}

#[tauri::command]
async fn create_ramdisk() -> String {
	println!("creating ramdisk");

	Command::new("sudo").args(["mkdir", "/mnt/ramdisk"]).output().unwrap();

	let output = Command::new("sudo").args(["mount", "-t", "ramfs", "-o", "size=250M", "ramfs", "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in Creating Ramdisk = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	let output = Command::new("sudo").args(["chmod", "777", "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in Creating Ramdisk = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//make the target dir for encrypted payload to or from SD cards
	Command::new("mkdir").args(["/mnt/ramdisk/sensitive"]).output().unwrap();

	format!("SUCCESS in Creating Ramdisk")
}

#[tauri::command]
fn read_cd() -> std::string::String {
    // sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
    let config_file = "/media/".to_string()+&get_user()+"/CDROM/config.txt";
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
            println!("read line: {}={}", n, v);
        }
    }
    format!("{}", contents)
}

#[tauri::command]
async fn combine_shards() -> String {
	println!("combining shards in /mnt/ramdisk/shards");
	let output = Command::new("bash")
		.args(["/home/".to_string()+&get_user()+"/scripts/combine-shards.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn async_write(name: &str, value: &str) -> Result<String, String> {
    write(name.to_string(), value.to_string());
    println!("{}", name);
    Ok(format!("completed with no problems"))
}

#[tauri::command]
async fn mount_internal() -> String {
	println!("mounting internal storage and symlinking .bitcoin dirs");
	let output = Command::new("bash")
		.args(["/home/".to_string()+&get_user()+"/scripts/mount-internal.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn install_sd_deps() -> String {
	println!("installing deps required by SD card");
	//these are required on all 7 SD cards
	// let output = Command::new("sudo").args(["add-apt-repository", "-y", "universe"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }

	// let output = Command::new("sudo").args(["apt", "update"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }

	// //download wodim
	// let output = Command::new("sudo").args(["apt", "install", "-y", "wodim"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }
	// //download shamir secret sharing library
	// let output = Command::new("sudo").args(["apt", "install", "ssss"]).output().unwrap();
	// if !output.status.success() {
    // 	// Function Fails
    // 	return format!("ERROR in installing SD dependencies = {}", std::str::from_utf8(&output.stderr).unwrap());
    // }

	//install sd dependencies for genisoimage
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing genisoimage {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for ssss
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/ssss_0.5-5_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing ssss {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for wodim
	let output = Command::new("sudo").args(["apt", "install" &(get_home()+"/dependencies/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing wodim {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	format!("SUCCESS in installing SD dependencies")
}

#[tauri::command]
async fn refresh_setup_cd() -> String {
	//create iso from setupCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/setupCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR refreshing setupCD with genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR refreshing setupCD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//burn setupCD iso to the setupCD
	let output = Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/setupCD.iso"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with burning iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in refreshing setupCD")
}

#[tauri::command]
async fn distribute_shards_sd2() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard2.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd2 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard10.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd2 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 2")
}

#[tauri::command]
async fn distribute_shards_sd3() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard3.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd3 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard9.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd3 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 3")
}

#[tauri::command]
async fn distribute_shards_sd4() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard4.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd4 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard8.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd4 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 4")
}

#[tauri::command]
async fn distribute_shards_sd5() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard5.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd5 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 5")
}

#[tauri::command]
async fn distribute_shards_sd6() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard6.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd6 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 6")
}

#[tauri::command]
async fn distribute_shards_sd7() -> String {
	//create local shards dir
	Command::new("mkdir").args([&("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard7.txt", &("/home/".to_string()+&get_user()+"/shards")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in distributing shards to sd7 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 7")
}

//deprecated
#[tauri::command]
async fn create_descriptor() -> String {
	println!("creating descriptor from 7 xpubs");
	let output = Command::new("bash")
		.args(["/home/".to_string()+&get_user()+"/scripts/create-descriptor.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

//deprecated
#[tauri::command]
async fn copy_descriptor() -> String {
	fs::copy("/mnt/ramdisk/CDROM/descriptor.txt", "/mnt/ramdisk/sensitive/descriptor.txt");
	format!("completed with no problems")
	
}

#[tauri::command]
async fn create_backup(number: String) -> String {
	println!("creating backup directory of the current SD");
		//make backup dir for iso
		Command::new("mkdir").args(["/mnt/ramdisk/backup"]).output().unwrap();
		//Copy shards to backup
		let output = Command::new("cp").args(["-r", &("/home/".to_string()+&get_user()+"/shards"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in creating backup with copying shards = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//Copy sensitive dir
		let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/encrypted.gpg"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in creating backup with copying sensitive dir= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//copy config
		let output = Command::new("cp").args([&("/home/".to_string()+&get_user()+"/config.txt"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in creating backup with copying config.txt= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//create .iso from backup dir
		let output = Command::new("genisoimage").args(["-r", "-J", "-o", &("/mnt/ramdisk/backup".to_string()+&number+".iso"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			// Function Fails
			return format!("ERROR in creating backup with creating iso= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	
		format!("SUCCESS in creating backup of current SD")
}

#[tauri::command]
async fn make_backup(number: String) -> String {
	println!("making backup iso of the current SD and burning to CD");
	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in making backup with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn setupCD iso to the backup CD
	let output = Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", &("/mnt/ramdisk/backup".to_string()+&number+".iso")]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in making backup with burning iso to CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in making backup of current SD")
}

#[tauri::command]
async fn start_bitcoind() -> String {
	println!("starting the bitcoin daemon");
	let output = Command::new(&("/home/".to_string()+&get_user()+"/bitcoin-23.0/bin/bitcoind")).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in starting bitcoin daemon = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in starting bitcoin daemon")
}

#[tauri::command]
async fn start_bitcoind_network_off() -> String {
	println!("starting the bitcoin daemon with networking disabled");
	let output = Command::new(&("/home/".to_string()+&get_user()+"/bitcoin-23.0/bin/bitcoind")).args(["-networkactive=0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in starting bitcoin daemon with networking disabled = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in starting bitcoin daemon with networking disabled")
}

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
async fn create_recovery_cd() -> String {
	println!("creating recovery CD for manual decrypting");
	//create transferCD config
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=transfercd" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in creating recovery CD, with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//collect shards from SD card for export to transfer CD
	let output = Command::new("cp").args(["-R", &("/media/".to_string()+&get_user()+"/shards"), "/mnt/ramdisk/CDROM/shards"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in creating recovery CD with copying shards from SD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	//create iso from transferCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR creating recovery CD with creating ISO = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn transferCD iso to the transfer CD
	Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in creating recovery CD")
}

#[tauri::command]
async fn copy_recovery_cd() -> String {
	Command::new("mkdir").args(["/mnt/ramdisk/shards"]).output().unwrap();
	copy_shards_to_ramdisk();
	format!("success in copying recovery CD")
}

#[tauri::command]
async fn calculate_number_of_shards_cd() -> u32 {
	let mut x = 0;
    for file in fs::read_dir("/media/".to_string()+&get_user()+"/CDROM/shards").unwrap() {
		x = x + 1;
	}
	return x;
}

#[tauri::command]
async fn calculate_number_of_shards_ramdisk() -> u32 {
	let mut x = 0;
    for file in fs::read_dir("/mnt/ramdisk/CDROM/shards").unwrap() {
		x = x + 1;
	}
	return x;
}


//broken
#[tauri::command]
async fn collect_shards() -> String {
	println!("collecting shards");
	//create transferCD target dir
	Command::new("mkdir").args(["--parents", "/mnt/ramdisk/CDROM/shards"]).output().unwrap();
	//stage transferCD target dir with current CD content
	let output = Command::new("cp").args(["-R", &("/media/".to_string()+&get_user()+"/CDROM"), "/mnt/ramdisk"]).output().unwrap();
	if !output.status.success() {
    	// Function Fails
    	return format!("ERROR in collecting shards with copying CDROM contents = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

    
	//create transferCD config
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=transfercd" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in converting to transfer CD, with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//this entire function is currently broken until a solution for the below recursive copy is discovered
	//collect shards from sd card for export to transfer CD
	//cp -r /home/$USER/shards/asterisk /mnt/ramdisk/CDROM/shards

	//maybe use this from copy_recovery_cd if needed
	// Command::new("mkdir").args(["/mnt/ramdisk/shards"]).output().unwrap();
	// copy_shards_to_ramdisk();

	//create iso from transferCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with creating ISO = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn transferCD iso to the transfer CD
	Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in collecting shards")
	
}

#[tauri::command]
async fn convert_to_transfer_cd() -> String {
	println!("converting completed recovery cd to transfer cd with masterkey");
	//create transferCD target dir
	Command::new("mkdir").args(["/mnt/ramdisk/CDROM"]).output().unwrap();
	//create transferCD config
	let file = File::create("/mnt/ramdisk/CDROM/config.txt").unwrap();
	let output = Command::new("echo").args(["type=transfercd" ]).stdout(file).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in converting to transfer CD, with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//this is a deprecated script as masterkey no longer lives in /ramdisk but instead lives in /ramdisk/CDROM. Revise. 
	//collect masterkey from cd dump and prepare for transfer to transfercd
	let output = Command::new("cp").args(["/mnt/ramdisk/masterkey", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in coverting to transfer CD, with copying masterkey = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//create iso from transferCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in converting to transfer CD, with copying masterkey = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("wodim").args(["-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR converting to transfer CD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn transferCD iso to the transfer CD
	let output = Command::new("wodim").args(["dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR refreshing setupCD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("eject").args(["/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		// Function Fails
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in converting to transfer CD")
}


// fn test() {

// 	println!("obtaining pid");
// 	//obtain pid
// 	let file = "./pid.txt";
// 	let pid = match fs::read_to_string(file){
// 		Ok(data) => data.replace("\n", ""),
// 		Err(err) => return println!("error {}", err.to_string())
// 	};
// 	println!("pid = {}", pid);
	
// 	println!("killing pid");
// 	//kill pid
// 	let output = Command::new("kill").args(["-9", &pid]).output().unwrap();
// 	if !output.status.success() {
// 		// Function Fails
// 		println!("ERROR in init iso with killing pid = {}", std::str::from_utf8(&output.stderr).unwrap());
// 	}
	
// }

fn main() {
	// test();
	let user_pass: bdk::blockchain::rpc::Auth = bdk::blockchain::rpc::Auth::UserPass{username: "rpcuser".to_string(), password: "477028".to_string()};
    let config: RpcConfig = RpcConfig {
	    url: "127.0.0.1:8332".to_string(),
	    auth: user_pass,
	    network: bdk::bitcoin::Network::Bitcoin,
	    wallet_name: "wallet_name".to_string(),
	    sync_params: None,
	};
  	tauri::Builder::default()
  	.manage(TauriState(Mutex::new(config), Mutex::new("".to_string()), Mutex::new("".to_string()), Mutex::new("".to_string())))
  	.invoke_handler(tauri::generate_handler![
        test_function,
        create_bootable_usb,
        create_setup_cd,
        read_cd,
        copy_cd_to_ramdisk,
        init_iso,
        async_write,
        read,
        combine_shards,
        mount_internal,
        create_ramdisk,
        packup,
        unpack,
        install_sd_deps,
        refresh_setup_cd,
        distribute_shards_sd2,
        distribute_shards_sd3,
        distribute_shards_sd4,
        distribute_shards_sd5,
        distribute_shards_sd6,
        distribute_shards_sd7,
        create_descriptor,
        copy_descriptor,
        create_backup,
        make_backup,
        start_bitcoind,
        start_bitcoind_network_off,
        check_for_masterkey,
        create_recovery_cd,
        copy_recovery_cd,
        calculate_number_of_shards_cd,
		calculate_number_of_shards_ramdisk,
        collect_shards,
        convert_to_transfer_cd,
		generate_store_key_pair,
		recover_key_pair,
        ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}