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

//import functions from helper
mod helper;
use helper::{get_user};

// file paths for this script and create_bootable_usb will need to change for prod
//these paths assume the user is compiling the application with cargo run inside ~/arctica
#[tauri::command]
pub async fn init_iso() -> String {
	println!("obtaining & creating modified ubuntu iso");

	println!("removing stale writable");
	//remove writable if exists, developer failsafe
	Command::new("sudo").args(["rm", "-r", "-f", &("/media/".to_string()+&get_user()+"/writable")]).output().unwrap();

	println!("unmounting stale writable & unbuntu mount if appropriate");
	//remove stale mount points if user has started arctica before
	Command::new("sudo").args(["umount", &("/media/".to_string()+&get_user()+"/Ubuntu 22.04.1 LTS amd64")]).output().unwrap();
	Command::new("sudo").args(["umount", &("/media/".to_string()+&get_user()+"/writable")]).output().unwrap();

	println!("downloading kvm dependencies");
	//download KVM deps
	Command::new("sudo").args(["apt-get", "-y", "install", "qemu-system-x86", "qemu-kvm", "libvirt-clients", "libvirt-daemon-system", "bridge-utils"]).output().unwrap();
	
	//obtain mkusb deps, 
	Command::new("sudo").args(["add-apt-repository", "-y", "universe"]).output().unwrap();
	Command::new("sudo").args(["add-apt-repository", "-y", "ppa:mkusb/ppa"]).output().unwrap();
	Command::new("sudo").args(["apt", "-y", "update"]).output().unwrap();
	Command::new("sudo").args(["apt", "install", "-y", "mkusb"]).output().unwrap();
	Command::new("sudo").args(["apt", "install", "-y", "usb-pack-efi"]).output().unwrap();


	//download dependencies required on each SD card
	Command::new("sudo").args(["apt", "update"]).output().unwrap();
	Command::new("sudo").args(["apt", "download", "wodim", "genisoimage", "ssss"]).output().unwrap();

	//check if ubuntu iso & bitcoin core already exists, and if no, obtain
	//NOTE: this currently checks the arctica repo but this will change once refactor is finished and user can run binary on host machine 
	println!("obtaining ubuntu iso and bitcoin core if needed");
	let a = std::path::Path::new("./ubuntu-22.04.1-desktop-amd64.iso").exists();
	let b = std::path::Path::new("./bitcoin-24.0.1-x86_64-linux-gnu.tar.gz").exists();
	if a == false{
		let output = Command::new("wget").args(["-O", "ubuntu-22.04.1-desktop-amd64.iso", "http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in init iso with downloading ubuntu iso = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	if b == false{
		let output = Command::new("wget").args(["https://bitcoincore.org/bin/bitcoin-core-24.0.1/bitcoin-24.0.1-x86_64-linux-gnu.tar.gz"]).output().unwrap();
		if !output.status.success() {
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
	//modify ubuntu iso to have a shorter timeout at boot screen
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
	//fallocate persistent iso, creates a 7GB image. Image size determines final storage space allocated to writable
	let output = Command::new("fallocate").args(["-l", "15GiB", "persistent-ubuntu.iso"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with fallocate persistent iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("booting iso with kvm");
	//boot kvm to establish persistence
	let output = Command::new("kvm").args(["-m", "2048", &(get_home()+"/arctica/persistent-ubuntu.iso"), "-daemonize", "-pidfile", "pid.txt", "-cpu", "host", "-display", "none"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with kvm = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("sleeping for 200 seconds");
	// sleep for 250 seconds
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
		return format!("ERROR in init iso with killing pid = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("mount persistent iso");
	//mount persistent iso at /media/$USER/writable/upper/
	let output = Command::new("udisksctl").args(["loop-setup", "-f", &(get_home()+"/arctica/persistent-ubuntu.iso")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with mounting persistent iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("sleep for 2 seconds");
	// sleep for 2 seconds
	Command::new("sleep").args(["2"]).output().unwrap();

	println!("opening file permissions for persistent dir");
	//open file permissions for persistent directory
	let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with opening file permissions of persistent dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("Making dependencies directory");
	//make dependencies directory
	Command::new("mkdir").args([&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();

	println!("Copying dependencies");
	//copying over dependencies genisoimage
	let output = Command::new("cp").args([&(get_home()+"/arctica/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//copying over dependencies ssss
	let output = Command::new("cp").args([&(get_home()+"/arctica/ssss_0.5-5_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying ssss = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//copying over dependencies wodim
	let output = Command::new("cp").args([&(get_home()+"/arctica/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/dependencies")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying wodim = {}", std::str::from_utf8(&output.stderr).unwrap());
	}


	println!("copying arctica binary");
	//copy over artica binary and make executable
	let output = Command::new("cp").args([&(get_home()+"/arctica/target/debug/app"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/arctica")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying arctica binary = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	println!("copying arctica icon");
	let output = Command::new("cp").args([&(get_home()+"/arctica/icons/arctica.jpeg"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/arctica.jpeg")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying binary jpeg = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	println!("making arctica a .desktop file");
	let output = Command::new("sudo").args(["cp", &(get_home()+"/arctica/shortcut/Arctica.desktop"), &("/media/".to_string()+&get_user()+"/writable/upper/usr/share/applications/Arctica.desktop")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying arctica.desktop = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//keeping this commented out for dev work due to regular binary swapping
	// println!("make arctica autostart at boot");
	// Command::new("mkdir").args([&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.config/autostart")]).output().unwrap();
	// let output = Command::new("sudo").args(["cp", &(get_home()+"/arctica/shortcut/Arctica.desktop"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.config/autostart")]).output().unwrap();
	// if !output.status.success() {
	// 	return format!("ERROR in init iso with copying arctica.desktop = {}", std::str::from_utf8(&output.stderr).unwrap());
	// }

	
	println!("making arctica binary an executable");
	//make the binary an executable file
	let output = Command::new("sudo").args(["chmod", "+x", &("/media/".to_string()+&get_user()+"/writable/upper/usr/share/applications/Arctica.desktop")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with making binary executable = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("copying scripts library");
	//copy over scripts directory and its contents. 
	let output = Command::new("cp").args(["-r", &(get_home()+"/arctica/scripts"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with copying scripts dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("extracting bitcoin core");
	//extract bitcoin core
	let output = Command::new("tar").args(["-xzf", &(get_home()+"/arctica/bitcoin-24.0.1-x86_64-linux-gnu.tar.gz"), "-C", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with extracting bitcoin core = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("create target device .bitcoin dir");
	//create target device .bitcoin dir
	let output = Command::new("mkdir").args([&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.bitcoin")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso with making target .bitcoin dir = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	println!("create bitcoin.conf on target device");
	//create bitcoin.conf on target device
	let file = File::create(&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/.bitcoin/bitcoin.conf")).unwrap();
	let output = Command::new("echo").args(["-e", "rpcuser=rpcuser\nrpcpassword=477028"]).stdout(file).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in init iso, with creating bitcoin.conf = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let start_time = Command::new("date").args(["+%s"]).output().unwrap();
	let start_time_output = std::str::from_utf8(&start_time.stdout).unwrap();
	println!("capturing and storing current unix timestamp");
	//capture and store current unix timestamp
	let mut fileRef = match std::fs::File::create(&("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/start_time")) {
		Ok(file) => file,
		Err(err) => return format!("Could not create start time file"),
	};
	fileRef.write_all(&start_time_output.to_string().as_bytes());

	format!("SUCCESS in init_iso")
}