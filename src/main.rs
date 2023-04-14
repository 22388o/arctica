#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

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
use helper::{get_user, print_rust, type_of, get_home, is_dir_empty, 
	copy_shards_to_ramdisk, write, check_cd_mount, get_uuid, generate_keypair, 
	store_string, store_psbt, build_high_descriptor, build_med_descriptor,
	build_low_descriptor, get_descriptor_checksum, unix_to_block_height,
	import_descriptor, create_wallet, retrieve_start_time, retrieve_start_time_integer
};

//local module imports
// mod install_module;
// use install_module::{init_iso};


// std::env::set_var("RUST_LOG", "bitcoincore_rpc=debug");

struct TauriState(Mutex<Option<Client>>);


#[tauri::command]
//current the config currently in $HOME
//conditional logic that determines application state is set by the front end after reading is completed
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

#[tauri::command]
//generates a public and private key pair and stores them as a text file
async fn generate_store_key_pair(number: String) -> String {
	//number corresponds to currentSD here and is provided by the front end
	let private_key_file = "/mnt/ramdisk/sensitive/private_key".to_string()+&number;
	let public_key_file = "/mnt/ramdisk/sensitive/public_key".to_string()+&number;

    let (xpriv, xpub) = match generate_keypair() {
		Ok((xpriv, xpub)) => (xpriv, xpub),
		Err(err) => return "ERROR could not generate keypair: ".to_string()+&err.to_string()
	}; 

	match store_string(xpriv.to_string()+"/*", &private_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store private key: ".to_string()+&err
	}
	match store_string(xpub.to_string()+"/*", &public_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store public key: ".to_string()+&err
	}
	//make the pubkey dir in the setupCD staging area if it does not already exist
	let a = std::path::Path::new("/mnt/ramdisk/CDROM/pubkeys").exists();
    if a == false{
		let output = Command::new("mkdir").args(["--parents", "/mnt/ramdisk/CDROM/pubkeys"]).output().unwrap();
		if !output.status.success() {
		return format!("ERROR in creating /mnt/ramdisk/CDROM/pubkeys dir {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	//copy public key to setupCD dir
	let output = Command::new("cp").args([&("/mnt/ramdisk/sensitive/public_key".to_string()+&number), "/mnt/ramdisk/CDROM/pubkeys"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in generate store key pair with copying pubkey= {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS generated and stored Private and Public Key Pair")
}

//this function simulates the creation of a time machine key. Eventually this creation will be performed by the BPS and 
//the pubkeys will be shared with the user instead. 4 Time machine Keys are needed so this function will be run 4 times in total.
//eventually these will need to be turned into descriptors and we will need an encryption scheme for the descriptors/keys that will be held by the BPS so as not to be privacy leaks
//decryption key will be held within encrypted tarball on each SD card
#[tauri::command]
async fn generate_store_simulated_time_machine_key_pair(number: String) -> String {
	//make the time machine key dir in the setupCD staging area if it does not already exist
	let a = std::path::Path::new("/mnt/ramdisk/CDROM/timemachinekeys").exists();
    if a == false{
		let output = Command::new("mkdir").args(["--parents", "/mnt/ramdisk/CDROM/timemachinekeys"]).output().unwrap();
		if !output.status.success() {
		return format!("ERROR in creating /mnt/ramdisk/CDROM/timemachinekeys dir {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}

	//number param is provided by the front end
	let private_key_file = "/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&number;
	let public_key_file = "/mnt/ramdisk/CDROM/timemachinekeys/time_machine_public_key".to_string()+&number;
	let (xpriv, xpub) = match generate_keypair() {
		Ok((xpriv, xpub)) => (xpriv, xpub),
		Err(err) => return "ERROR could not generate keypair: ".to_string()+&err.to_string()
	};
	match store_string(xpriv.to_string()+"/*", &private_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store private key: ".to_string()+&err
	}
	match store_string(xpub.to_string()+"/*", &public_key_file) {
		Ok(_) => {},
		Err(err) => return "ERROR could not store public key: ".to_string()+&err
	}

	//copy public key to setupCD dir
	let output = Command::new("cp").args([&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_public_key".to_string()+&number), "/mnt/ramdisk/CDROM/pubkeys"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in generate store key pair with copying pubkey= {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	format!("SUCCESS generated and stored Private and Public Key Pair")
}

#[tauri::command]
//get a new address
//accepts "low", "immediate", and "delayed" as a param
//equivalent to... Command::new("/bitcoin-24.0.1/bin/bitcoin-cli").args([&("-rpcwallet=".to_string()+&(wallet.to_string())+"_wallet"), "getnewaddress"])
//must be done with client url param URL=<hostname>/wallet/<wallet_name>
async fn get_address(wallet: String, sdcard:String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	//address labels can be added here
	let address_type = Some(AddressType::Bech32);
	let address = match Client.get_new_address(None, address_type){
		Ok(addr) => addr,
		Err(err) => return Ok(format!("{}", err.to_string()))
	};
	Ok(format!("{}", address))
}

#[tauri::command]
//calculate the current balance of the tripwire wallet
async fn get_balance(wallet:String, sdcard:String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	let balance = match Client.get_balance(None, Some(true)){
		Ok(bal) => {
			//split string into a vec and extract the number only without the BTC unit
			let bal_output = bal.to_string();
			let split = bal_output.split(' ');
			let bal_vec: Vec<_> = split.collect();
			return Ok(bal_vec[0].to_string())
			
		},
		Err(err) => return Ok(format!("{}", err.to_string()))
	};
}



#[derive(Serialize)]
struct CustomTransaction {
	id: i32,
    info: CustomWalletTxInfo,
    detail: CustomGetTransactionResultDetail,
    trusted: Option<bool>,
    comment: Option<String>,
}

#[derive(Serialize)]
struct CustomWalletTxInfo {
    confirmations: i32,
    blockhash: Option<String>,
    blockindex: Option<usize>,
    blocktime: Option<u64>,
    blockheight: Option<u32>,
    txid: String,
    time: u64,
    timereceived: u64,
    bip125_replaceable: String,
    wallet_conflicts: Vec<String>,
}

#[derive(Serialize)]
struct CustomGetTransactionResultDetail {
    address: Option<String>,
    category: String,
    amount: i64,
    label: Option<String>,
    vout: u32,
    fee: Option<i64>,
    abandoned: Option<bool>,
}





#[tauri::command]
//retrieve the current transaction history for the immediate wallet
async fn get_transactions(wallet: String, sdcard:String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
   let transactions: Vec<ListTransactionResult> = match Client.list_transactions(None, None, None, Some(true)) {
	Ok(tx) => tx,
	Err(err) => return Ok(format!("{}", err.to_string()))
   };

   if transactions.is_empty() {
	return Ok(format!("empty123321"))
   }
   else{
	let mut custom_transactions: Vec<CustomTransaction> = Vec::new();
	let mut x = 0;
   
	for tx in transactions {
		let custom_tx = CustomTransaction {
			id: x,
			info: CustomWalletTxInfo {
				confirmations: tx.info.confirmations,
				blockhash: tx.info.blockhash.map(|hash| hash.to_string()),
				blockindex: tx.info.blockindex,
				blocktime: tx.info.blocktime,
				blockheight: tx.info.blockheight,
				txid: tx.info.txid.to_string(),
				time: tx.info.time,
				timereceived: tx.info.timereceived,
				bip125_replaceable: match tx.info.bip125_replaceable {
					Bip125Replaceable::Yes => "Yes".to_string(),
					Bip125Replaceable::No => "No".to_string(),
					Bip125Replaceable::Unknown => "Unknown".to_string(),
				},
				wallet_conflicts: tx.info.wallet_conflicts.into_iter().map(|c| c.to_string()).collect(),
			},
			detail: CustomGetTransactionResultDetail {
				address: tx.detail.address.as_ref().map(|addr| addr.to_string()),
				category: match tx.detail.category {
				 GetTransactionResultDetailCategory::Send => "Send".to_string(),
				 GetTransactionResultDetailCategory::Receive => "Receive".to_string(),
				 GetTransactionResultDetailCategory::Generate => "Generate".to_string(),
				 GetTransactionResultDetailCategory::Immature => "Immature".to_string(),
				 GetTransactionResultDetailCategory::Orphan => "Orphan".to_string(),
			 }, 
				amount: tx.detail.amount.to_sat(),
				label: tx.detail.label,
				vout: tx.detail.vout,
				fee: tx.detail.fee.map_or_else(|| None, |x| Some(x.to_sat())),
				abandoned: tx.detail.abandoned,
			},
			trusted: tx.trusted,
			comment: tx.comment,
		};

		//check if the address is owned by the wallet, if so, assume change input/output and hide from the display
		let addr: Address = tx.detail.address.unwrap();
		let ismine_res = Client.get_address_info(&addr);
		let ismine = match ismine_res{
			Ok(res)=>res,
			Err(err)=>return Ok(format!("{}", err.to_string()))
		};
		// if ismine.is_mine == Some(false)||Some(true){
			custom_transactions.push(custom_tx);
			x += 1;
		// }
		// else{
		// 	continue
		// }
		
	}
	
	let json_string = serde_json::to_string(&custom_transactions).unwrap();
	println!("{}", json_string);
 
	Ok(format!("{}", json_string))
   }


}


//TODO send flow
//on SD 1
//generate PSBT
//process PSBT (SIGN)
//export processed PSBT
//on SD 2
//import signed PSBTs
//process PSBT (SIGN)
//export processed PSBT
//on SD 1 import processed PSBT
//combine psbts
//finalize psbt
//sendrawtransaction

#[tauri::command]
//generate a PSBT for the immediate wallet
//will require additional logic to spend when under decay threshold
//currently only generates a PSBT for Key 1 and Key 2, which are SD 1 and SD 2 respectively
async fn generate_psbt(wallet: String, sdcard:String, recipient: &str, amount: f64, fee: u64) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
   //create the directory where the PSBT will live if it does not exist
   let a = std::path::Path::new("/mnt/ramdisk/psbt").exists();
   if a == false{
       //remove the stale dir
       let output = Command::new("mkdir").args(["/mnt/ramdisk/psbt"]).output().unwrap();
       if !output.status.success() {
       return Ok(format!("ERROR in creating /mnt/ramdisk/psbt dir {}", std::str::from_utf8(&output.stderr).unwrap()));
       }
   }
   //declare the destination for the PSBT file
   let file_dest = "/mnt/ramdisk/psbt/psbt".to_string();

   //define change address type
   let address_type = Some(AddressType::Bech32);

   //obtain a change address
   let change_address = match Client.get_new_address(None, address_type){
	   Ok(addr) => addr,
	   Err(err) => return Ok(format!("{}", err.to_string()))
   };



   //below code block is for trying to use bitcoincore_rpc crate to generate psbt, method is currently bugged
   //alternatively going to do the below with Command::new() and will return to this method when it is fixed

//    //define the inputs struct, leave empty for dynamic input selection
// 	let inputs = vec![];

// 	//define outputs hashmap
//    let mut outputs = HashMap::new();

//    //add the recipient to the outputs hashmap
//    outputs.insert(
// 	String::from_str(recipient).unwrap(),
// 	Amount::from_sat(amount),
//    );

//    //add the change address to the outputs hashmap
//    outputs.insert(
// 	change_address.to_string(),
// 	Amount::from_btc(0),
//    );

//    //declare the options struct with the default params
//    let mut options = WalletCreateFundedPsbtOptions::default();

//    //set the fee rate
//    	// options.fee_rate = Some(Amount::from_sat(fee));

//    //build the transaction
//   let psbt_result = Client.wallet_create_funded_psbt(
// 	&inputs, //no inputs specified
// 	&outputs, //outputs specified in the outputs struct
// 	None, //no locktime specified
// 	Some(options), //options specified in the options struct
// 	None, //no bip32derivs specified
//   	);

// 	//obtain the result of wallet_create_funded_psbt
// 	let psbt_res = match psbt_result{
// 		Ok(psbt)=> psbt,
// 		Err(err)=> return Ok(format!("{}", err.to_string()))
		
// 	};
	
// 	//decode the psbt
// 	let psbt = decode(&psbt_res.psbt).unwrap();

// 	//convert the decoded psbt to a string
// 	let psbt_str = to_string(&psbt).unwrap();

let json_input = json!([]);


let mut json_output = json!([{
	recipient: amount
}]);

let change_arg = json!({
	"changeAddress": change_address
});

let locktime = "0";

let psbt_output = Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoin-cli"))
.args([&("-rpcwallet=".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), 
"walletcreatefundedpsbt", 
&json_input.to_string(), //empty array
&json_output.to_string(), //receive address & output amount
&locktime, //locktime should always be 0
&change_arg.to_string() ]) //manually providing change address
.output()
.unwrap();
if !psbt_output.status.success() {
	return Ok(format!("ERROR in generating PSBT = {}", std::str::from_utf8(&psbt_output.stderr).unwrap()));
}


let psbt_str = String::from_utf8(psbt_output.stdout).unwrap();

let psbt: WalletCreateFundedPsbtResult = match serde_json::from_str(&psbt_str) {
	Ok(psbt)=> psbt,
	Err(err)=> return Ok(format!("{}", err.to_string()))
};

	// sign the PSBT
	let signed_result = Client.wallet_process_psbt(
		&psbt.psbt,
		Some(true),
		None,
		None,
	);

	let signed = match signed_result{
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
		
	};
	

   //store the transaction as a file
       match store_psbt(&signed, file_dest) {
       Ok(_) => {},
       Err(err) => return Err("ERROR could not store PSBT: ".to_string()+&err)
       };

   Ok(format!("PSBT: {:?}", signed))
}




#[tauri::command]
//for testing only
async fn test_function() -> String {
	format!("testing")
}


// file paths for this script and create_bootable_usb will need to change for prod
//these paths assume the user is compiling the application with cargo run inside ~/arctica
#[tauri::command]
async fn init_iso() -> String {
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

//initial flash of all 7 SD cards
//creates a bootable usb stick or SD card that will boot into an ubuntu live system when inserted into a computer
#[tauri::command]
async fn create_bootable_usb(number: String, setup: String) -> String {
	//write device type to config, values provided by front end
    write("type".to_string(), "sdcard".to_string());
	//write sd number to config, values provided by front end
    write("sdNumber".to_string(), number.to_string());
	//write current setup step to config, values provided by front end
    write("setupStep".to_string(), setup.to_string());
	println!("creating bootable ubuntu device writing config...SD {} Setupstep {}", number, setup);

	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//remove old config from iso
	Command::new("sudo").args(["rm", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/config.txt")]).output().unwrap();
	//copy new config
	let output = Command::new("sudo").args(["cp", &(get_home()+"/config.txt"), &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in creating bootable with copying current config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//open file permissions for config
	let output = Command::new("sudo").args(["chmod", "777", &("/media/".to_string()+&get_user()+"/writable/upper/home/ubuntu/config.txt")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in creating bootable with opening file permissions = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//remove current working config from local
	let output = Command::new("sudo").args(["rm", &(get_home()+"/config.txt")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in creating bootable with removing current working config = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//burn iso with mkusb
	let mkusb_child = Command::new("printf").args(["%s\n", "n", "y", "g", "y"]).stdout(Stdio::piped()).spawn().unwrap();
	println!("received stdout, piping to mkusb");
	let mkusb_child_two = Command::new("mkusb").args([&(get_home()+"/arctica/persistent-ubuntu.iso")]).stdin(Stdio::from(mkusb_child.stdout.unwrap())).stdout(Stdio::piped()).spawn().unwrap();
	println!("mkusb finished creating output");
	let output = mkusb_child_two.wait_with_output().unwrap();
	if !output.status.success() {
		return format!("ERROR in creating bootable with mkusb = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in creating bootable device")
}

#[tauri::command]
async fn create_setup_cd() -> String {
	println!("creating setup CD");
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	//install sd dependencies for genisoimage
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing genisoimage for create_setup_cd {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for ssss
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/ssss_0.5-5_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing ssss for create_setup_cd {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for wodim
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
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
	let output = Command::new("sudo").args(["cp", "/mnt/ramdisk/shards/shard1.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in copying shard1.txt in create setup CD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }
	let output = Command::new("sudo").args(["cp", "/mnt/ramdisk/shards/shard11.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in copying shard11.txt in create setup CD = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//remove stale shard file
	let output = Command::new("sudo").args(["rm", "/mnt/ramdisk/shards_untrimmed.txt"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in removing deprecated shards_untrimmed in create setup cd = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//stage setup CD dir with shards for distribution
	let output = Command::new("sudo").args(["cp", "-R", "/mnt/ramdisk/shards", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in copying shards to CDROM dir in create setup cd = {}", std::str::from_utf8(&output.stderr).unwrap());
    }

	//create iso from setupCD dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/setupCD.iso", "/mnt/ramdisk/CDROM"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR refreshing setupCD with genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	let output = Command::new("sudo").args(["wodim", "-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR refreshing setupCD with wiping CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//burn setupCD iso to the setupCD
	let output = Command::new("sudo").args(["wodim", "dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/setupCD.iso"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with burning iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	//eject the disc
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in Creating Setup CD")

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

//pack up and encrypt the contents of the sensitive directory in ramdisk into an encrypted directory on tpackuphe current SD card
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

//decrypt & unpack the contents of an encrypted directory on the current SD card into the sensitive directory in ramdisk
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

		//make the target dir for encrypted payload to or from SD cards
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

    for line in contents.split("\n") {
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() == 2 {
            let (n,v) = (parts[0],parts[1]);
            println!("read line: {}={}", n, v);
        }
    }
    format!("{}", contents)
	
	
}

//used to combine recovered shards into an encryption/decryption masterkey
#[tauri::command]
async fn combine_shards() -> String {
	println!("combining shards in /mnt/ramdisk/shards");
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
		let output = Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoin-cli")).args(["stop"]).output().unwrap();
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

#[tauri::command]
//install dependencies manually from files on each of the offline SD cards (2-7)
async fn install_sd_deps() -> String {
	println!("installing deps required by SD card");
	//these are required on all 7 SD cards
	//install sd dependencies for genisoimage
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/genisoimage_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing genisoimage {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for ssss
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/ssss_0.5-5_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing ssss {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	//install sd dependencies for wodim
	let output = Command::new("sudo").args(["apt", "install", &(get_home()+"/dependencies/wodim_9%3a1.1.11-3.2ubuntu1_amd64.deb")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in installing wodim {}", std::str::from_utf8(&output.stderr).unwrap());
	} 

	format!("SUCCESS in installing SD dependencies")
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

//The following "distribute_shards" fuctions are for distributing encryption key shards to each of the sd cards 2-7
#[tauri::command]
async fn distribute_shards_sd2() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard2.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd2 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard10.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd2 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 2")
}

#[tauri::command]
async fn distribute_shards_sd3() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard3.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd3 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard9.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd3 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 3")
}

#[tauri::command]
async fn distribute_shards_sd4() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard4.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd4 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard8.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd4 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 4")
}

#[tauri::command]
async fn distribute_shards_sd5() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard5.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd5 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 5")
}

#[tauri::command]
async fn distribute_shards_sd6() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard6.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd6 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 6")
}

#[tauri::command]
async fn distribute_shards_sd7() -> String {
	//create local shards dir
	Command::new("mkdir").args([&(get_home()+"/shards")]).output().unwrap();

	let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/shards/shard7.txt", &(get_home()+"/shards")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in distributing shards to sd7 = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in distributing shards to SD 7")
}

//create arctica descriptors
//High Descriptor is the time locked 5 of 11 with decay (4 keys will eventually go to BPS)
//Medium Descriptor is the 2 of 7 with decay
//Low Descriptor is the 1 of 7 and will be used for the tripwire
//acceptable params should be "1", "2", "3", "4", "5", "6", "7"
#[tauri::command]
async fn create_descriptor(sdcard: String) -> Result<String, String> {
   println!("creating descriptors from 7 xpubs & 4 time machine keys");
   //convert all 11 public_keys in the ramdisk to an array vector
   println!("creating key array");
   let mut key_array = Vec::new();
   //push the 7 standard public keys into the key_array vector
   println!("pushing 7 standard pubkeys into key array");
   for i in 1..=7{
       let key = fs::read_to_string(&("/mnt/ramdisk/CDROM/pubkeys/public_key".to_string()+&(i.to_string()))).expect(&("Error reading public_key from file".to_string()+&(i.to_string())));
       key_array.push(key);
       println!("pushed key");
   }

   //push the 4 time machine public keys into the key_array vector, only on SD 1.
	println!("pushing 4 time machine pubkeys into key array");
	for i in 1..=4{
		let key = fs::read_to_string(&("/mnt/ramdisk/CDROM/pubkeys/time_machine_public_key".to_string()+&(i.to_string()))).expect(&("Error reading time_machine_public_key from file".to_string()+&(i.to_string())));
		key_array.push(key);
		println!("pushed key");
	}
   


   println!("printing key array");
   println!("{:?}", key_array);

   //create the descriptors directory inside of ramdisk
   println!("Making descriptors dir");
   Command::new("mkdir").args(["/mnt/ramdisk/sensitive/descriptors"]).output().unwrap();

   //build the delayed wallet descriptor
   println!("building high descriptor");
   let high_descriptor = match build_high_descriptor(&key_array, &sdcard) {
	Ok(desc) => desc,
	Err(err) => return Err("ERROR could not build High Descriptor ".to_string()+&err)
   };
   let high_file_dest = &("/mnt/ramdisk/sensitive/descriptors/delayed_descriptor".to_string()+&sdcard.to_string()).to_string();
   //store the delayed wallet descriptor in the sensitive dir
   println!("storing high descriptor");
   match store_string(high_descriptor.to_string(), high_file_dest) {
       Ok(_) => {},
       Err(err) => return Err("ERROR could not store High Descriptor: ".to_string()+&err)
   };
   println!("creating delayed wallet");
   match create_wallet("delayed".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not create Delayed Wallet: ".to_string()+&err)
   };
   println!("importing delayed descriptor");
   match import_descriptor("delayed".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not import Delayed Descriptor: ".to_string()+&err)
   };
   

   //build the immediate wallet descriptor
   println!("building med descriptor");
   let med_descriptor = match build_med_descriptor(&key_array, &sdcard) {
	Ok(desc) => desc,
	Err(err) => return Err("ERROR could not build Immediate Descriptor ".to_string()+&err)
   };
   let med_file_dest = &("/mnt/ramdisk/sensitive/descriptors/immediate_descriptor".to_string()+&sdcard.to_string()).to_string();
   //store the immediate wallet descriptor in the sensitive dir
   println!("storing med descriptor");
   match store_string(med_descriptor.to_string(), med_file_dest) {
       Ok(_) => {},
       Err(err) => return Err("ERROR could not store Immediate Descriptor: ".to_string()+&err)
   };
   println!("creating immediate wallet");
   match create_wallet("immediate".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not create Immediate Wallet: ".to_string()+&err)
   };
   println!("importing immediate descriptor");
   match import_descriptor("immediate".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not import Immediate Descriptor: ".to_string()+&err)
   };

   //build the low security descriptor
   println!("building low descriptor");
   let low_descriptor = match build_low_descriptor(&key_array, &sdcard) {
	Ok(desc) => desc,
	Err(err) => return Err("ERROR could not build Low Descriptor ".to_string()+&err)
   };
   let low_file_dest = &("/mnt/ramdisk/sensitive/descriptors/low_descriptor".to_string()+&sdcard.to_string()).to_string();
   //store the low security descriptor in the sensitive dir
   println!("storing low descriptor");
   match store_string(low_descriptor.to_string(), low_file_dest) {
       Ok(_) => {},
       Err(err) => return Err("ERROR could not store Low Descriptor: ".to_string()+&err)
   };
   println!("creating low wallet");
   match create_wallet("low".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not create Low Wallet: ".to_string()+&err)
   };
   println!("importing low descriptor");
   match import_descriptor("low".to_string(), &sdcard){
	Ok(_) => {},
	Err(err) => return Err("ERROR could not import Low Descriptor: ".to_string()+&err)
   };

   println!("Success");
   Ok(format!("SUCCESS in creating descriptors"))

}

//Create a backup directory of the currently inserted SD card
#[tauri::command]
async fn create_backup(number: String) -> String {
	println!("creating backup directory of the current SD");
		//make backup dir for iso
		Command::new("mkdir").args(["/mnt/ramdisk/backup"]).output().unwrap();
		//Copy shards to backup
		let output = Command::new("cp").args(["-r", &(get_home()+"/shards"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in creating backup with copying shards = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//Copy sensitive dir
		let output = Command::new("cp").args([&(get_home()+"/encrypted.gpg"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in creating backup with copying sensitive dir= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//copy config
		let output = Command::new("cp").args([&(get_home()+"/config.txt"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in creating backup with copying config.txt= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//create .iso from backup dir
		let output = Command::new("genisoimage").args(["-r", "-J", "-o", &("/mnt/ramdisk/backup".to_string()+&number+".iso"), "/mnt/ramdisk/backup"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in creating backup with creating iso= {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	
		format!("SUCCESS in creating backup of current SD")
}

//make the existing backup directory into an iso and burn to the currently inserted CD/DVD
#[tauri::command]
async fn make_backup(number: String) -> String {
	println!("making backup iso of the current SD and burning to CD");
	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//wipe the CD
	Command::new("sudo").args(["umount", "/dev/sr0"]).output().unwrap();
	//we don't mind if this fails, CD-Rs will fail this script always
	Command::new("sudo").args(["wodim", "-v", "dev=/dev/sr0", "blank=fast"]).output().unwrap();

	//burn setupCD iso to the backup CD
	let output = Command::new("sudo").args(["wodim", "dev=/dev/sr0", "-v", "-data", &("/mnt/ramdisk/backup".to_string()+&number+".iso")]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in making backup with burning iso to CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in making backup of current SD")
}

//start bitcoin core daemon
#[tauri::command]
async fn start_bitcoind() -> String {
	//enable networking 
	//the only time this  block should be required is immediately following initial setup
	//networing is force disabled for key generation on all SD cards. 
	let output = Command::new("sudo").args(["nmcli", "networking", "on"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR disabling networking = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	let uuid = get_uuid();
	//mount internal drive if nvme
	if uuid == "ERROR in parsing /media/user" {
		return format!("Error in parsing /media/user to get uuid");
	}
	else if uuid == "none"{
		return format!("ERROR could not find a valid UUID in /media/$user");
	}else{
		let host = Command::new(&("ls")).args([&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home")]).output().unwrap();
		if !host.status.success() {
			return format!("ERROR in parsing /media/user/uuid/home {}", std::str::from_utf8(&host.stderr).unwrap());
		} 
		let host_user = std::str::from_utf8(&host.stdout).unwrap().trim();
		//check if walletdir exists and if not create it
		let a = std::path::Path::new("/mnt/ramdisk/sensitive/wallets").exists();
		if a == false {
			let output = Command::new("mkdir").args(["/mnt/ramdisk/sensitive/wallets"]).output().unwrap();
			if !output.status.success() {
				return format!("ERROR in starting bitcoin daemon with creating ../sensitive/wallets dir = {}", std::str::from_utf8(&output.stderr).unwrap());
			}
		}
		//start bitcoin daemon with proper datadir & walletdir path
		std::thread::spawn( ||{
			//redeclare dynamic vars within the new scope
			let uuid = get_uuid();
			let host = Command::new(&("ls")).args([&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home")]).output().unwrap();
			let host_user = std::str::from_utf8(&host.stdout).unwrap().trim();
			//spawn as a child process on a seperate thread, nullify the output
			Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoind"))
			.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), &("-datadir=/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin"), "-walletdir=/mnt/ramdisk/sensitive/wallets"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.stdin(Stdio::null())
			.spawn();
			});
		loop{
			//redeclare the Client object within the new scope
			let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
			let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
			//query getblockchaininfo
			match Client.get_blockchain_info(){
				//if a valid response is received...
				Ok(res) => {
					//sleep and continue the loop in the event that the chain is not synced
					let progress =  res.verification_progress; 
					if progress < 0.9999{
						std::thread::sleep(Duration::from_secs(5));
						continue;
					}
					//break the loop in the event the sync exceed 0.9999
					else{
						break;
					}
				},
				//error is returned when the daemon is still performing initial block db verification
				Err(error) => {
					//sleep and continue the loop
					std::thread::sleep(Duration::from_secs(5));
					continue;
				},
			};
			
		}
		format!("SUCCESS in starting bitcoin daemon")
	}
}

//start bitcoin core daemon with networking disabled
//this will prevent block sync
//use this function when starting core daemon on any offline device
#[tauri::command]
fn start_bitcoind_network_off() -> String {
	//disable networking
	let output = Command::new("sudo").args(["nmcli", "networking", "off"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR disabling networking = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	if !output.status.success() {
		return format!("ERROR disabling networking = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//check if walletdir exists and if not create it
	let a = std::path::Path::new("/mnt/ramdisk/sensitive/wallets").exists();
	if a == false {
		Command::new("mkdir").args(["/mnt/ramdisk/sensitive/wallets"]).output().unwrap();
		//start bitcoin daemon with networking inactive and proper walletdir path
		//spawn as a child process on a seperate thread, nullify the output
		std::thread::spawn( ||{
			Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoind"))
			.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), "-walletdir=/mnt/ramdisk/sensitive/wallets"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.stdin(Stdio::null())
			.spawn();
			});
	}
	else {
		//start bitcoin daemon with networking inactive and proper walletdir path
		//spawn as a child process on a seperate thread, nullify the output
		std::thread::spawn( ||{
			Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoind"))
			.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), "-walletdir=/mnt/ramdisk/sensitive/wallets"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.stdin(Stdio::null())
			.spawn();
			});
	}
	format!("SUCCESS in starting bitcoin daemon with networking disabled")
	}
	


#[tauri::command]
async fn stop_bitcoind() -> String {
	//start bitcoin daemon with networking inactive
	let output = Command::new(&(get_home()+"/bitcoin-24.0.1/bin/bitcoin-cli")).args(["stop"]).output().unwrap();
	if !output.status.success() {
		
		return format!("ERROR in stopping bitcoin daemon = {}", std::str::from_utf8(&output.stderr).unwrap());
	}

	format!("SUCCESS in stopping the bitcoin daemon")
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
//this fn is used to store decryption shards gathered from various SD cards to eventually be reconstituted into a masterkey when attempting to log in manually
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
	//collect shards from SD card for export to transfer CD
	let output = Command::new("cp").args(["-R", &(get_home()+"/shards"), "/mnt/ramdisk/CDROM/shards"]).output().unwrap();
	if !output.status.success() {
    	return format!("ERROR in creating recovery CD with copying shards from SD = {}", std::str::from_utf8(&output.stderr).unwrap());
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
    for file in fs::read_dir("/mnt/ramdisk/CDROM/shards").unwrap() {
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

//helper function, RPC command
// ./bitcoin-cli getdescriptorinfo '<descriptor>'
// analyze a descriptor and report a canonicalized version with checksum added
//acceptable params here are "low", "immediate", "delayed"
//this may not be useful for anything besides debugging on the fly
#[tauri::command]
async fn get_descriptor_info(wallet: String) -> String {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
	let desc: String = fs::read_to_string(&("/mnt/ramdisk/sensitive/descriptors/".to_string()+&(wallet.to_string())+"_descriptor")).expect("Error reading reading med descriptor from file");
	let desc_info = Client.get_descriptor_info(&desc).unwrap();
	format!("SUCCESS in getting descriptor info {:?}", desc_info)
}


#[tauri::command]
async fn load_wallet(wallet: String, sdcard: String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");

	//load the specified wallet
	Client.load_wallet(&(wallet.to_string()+"_wallet"+&(sdcard.to_string())));

	//parse list_wallets in a continuous loop to verify when rescan is completed
	loop{
		let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    	let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
		let list = Client.list_wallets().unwrap();
		let search_string = &(wallet.to_string()+"_wallet"+&(sdcard.to_string()));
		//listwallets returns the wallet name as expected...wallet is properly loaded and scanned
		if list.contains(&search_string){
			break;
		}
		//listwallets does not return the wallet name...wallet not yet loaded
		else{
			std::thread::sleep(Duration::from_secs(5));
			continue;
		}
	}
	Ok(format!("Success in loading {} wallet", wallet))
	}


#[tauri::command]
async fn get_blockchain_info() -> String {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
	let info = Client.get_blockchain_info();
	format!("Results: {:?}", info)
}

#[tauri::command]
async fn export_psbt(progress: String) -> String{
	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//create conf for CD
	let a = std::path::Path::new("/mnt/ramdisk/psbt/config.txt").exists();
	if a == false{
		let file = File::create(&("/mnt/ramdisk/psbt/config.txt")).unwrap();
		let output = Command::new("echo").args(["-e", &("psbt=".to_string()+&progress.to_string())]).stdout(file).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in export_psbt with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	let b = std::path::Path::new("/mnt/ramdisk/psbt/masterkey").exists();
	//copy over masterkey
	if b == false{
		let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/masterkey", "/mnt/ramdisk/psbt"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR in export_psbt with creating config = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	//create iso from psbt dir
	let output = Command::new("genisoimage").args(["-r", "-J", "-o", "/mnt/ramdisk/transferCD.iso", "/mnt/ramdisk/psbt"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR creating psbt iso with genisoimage = {}", std::str::from_utf8(&output.stderr).unwrap());
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
	//burn psbt iso to the transferCD
	let output = Command::new("sudo").args(["wodim", "dev=/dev/sr0", "-v", "-data", "/mnt/ramdisk/transferCD.iso"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with burning iso = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//eject the disc
	let output = Command::new("sudo").args(["eject", "/dev/sr0"]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR in refreshing setupCD with ejecting CD = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	format!("SUCCESS in Creating transferCD")
}

#[tauri::command]
async fn sign_psbt(wallet: String, sdcard: String, progress: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	//TODO
	//import the psbt from ramdisk (perhaps break this into a seperate function? maybe not because it has to be used within scope)...but potentially we should analyze before signing
	let psbt_str: String = fs::read_to_string("/mnt/ramdisk/CDROM/psbt").expect("Error reading PSBT from file");

	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//attempt to sign the tx
	let signed_result = Client.wallet_process_psbt(
		&psbt.psbt,
		Some(true),
		None,
		None,
	);
	let signed = match signed_result{
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//declare file dest
	let file_dest = "/mnt/ramdisk/CDROM/psbt".to_string();
	//remove stale psbt from /mnt/ramdisk/CDROM/psbt
	Command::new("sudo").args(["rm", "/mnt/ramdisk/CDROM/psbt"]).output().unwrap();
	//store the signed transaction as a file
	match store_psbt(&signed, file_dest) {
	Ok(_) => {},
	Err(err) => return Err("ERROR could not store PSBT: ".to_string()+&err)
	};
	//remove the stale config.txt
	Command::new("sudo").args(["rm", "/mnt/ramdisk/CDROM/config.txt"]).output().unwrap();
	let file = File::create(&("/mnt/ramdisk/CDROM/config.txt")).unwrap();
	let output = Command::new("echo").args(["-e", &("psbt=".to_string()+&progress.to_string())]).stdout(file).output().unwrap();
	if !output.status.success() {
		return Ok(format!("ERROR in sign_psbt with creating config = {}", std::str::from_utf8(&output.stderr).unwrap()));
	}

	Ok(format!("Reading PSBT from file: {:?}", signed))
}

#[tauri::command]
async fn finalize_psbt(wallet: String, sdcard: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	let psbt_str: String = fs::read_to_string("/mnt/ramdisk/CDROM/psbt").expect("Error reading PSBT from file");
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//finalize the tx
	let finalized_result = Client.finalize_psbt(
		&psbt.psbt,
		None,
	);
	let finalized = match finalized_result{
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
		
	};
	
	Ok(format!("Reading PSBT from file: {:?}", finalized))
}

#[tauri::command]
async fn broadcast_tx(wallet: String, sdcard: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	//read the psbt from the transfer CD
	let psbt_str: String = fs::read_to_string("/mnt/ramdisk/CDROM/psbt").expect("Error reading PSBT from file");
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//finalize the psbt
	let finalized_result = Client.finalize_psbt(
		&psbt.psbt,
		None,
	);
	let finalized = match finalized_result{
		Ok(tx)=> tx.hex.unwrap(),
		Err(err)=> return Ok(format!("{}", err.to_string()))	
	};

	//broadcast the tx
	let broadcast_result = Client.send_raw_transaction(&finalized[..]);

	let broadcast = match broadcast_result{
		Ok(tx)=> tx,
		Err(err)=> return Ok(format!("{}", err.to_string()))	
	};
	
	Ok(format!("Broadcasting Fully Signed TX: {:?}", broadcast))
}

//used to decode a fully signed TX...might be able to remove the
#[tauri::command]
async fn decode_raw_tx(wallet: String, sdcard: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let Client = bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+&sdcard.to_string()), auth).expect("could not connect to bitcoin core");
	//read the psbt from the transfer CD
	let psbt_str: String = fs::read_to_string("/mnt/ramdisk/CDROM/psbt").expect("Error reading PSBT from file");
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};

	let psbt_bytes = base64::decode(&psbt.psbt).unwrap();
	let psbtx: PartiallySignedTransaction = deserialize(&psbt_bytes[..]).unwrap();
	let unsigned_tx = psbtx.extract_tx();
	let hex_tx = serialize(&unsigned_tx);

	let decoded_result = Client.decode_raw_transaction(&hex_tx[..], None);

	let decoded = match decoded_result{
		Ok(result) => result,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};

	let clone = decoded.vout[0].clone();
	let address: String = clone.script_pub_key.address.unwrap().to_string();
	let amount = clone.value;

	// Calculate the total value of the transaction outputs
	let output_total: Amount = decoded
		.vout
		.iter()
		.filter_map(|output| Some(output.value))
		.sum();
	
	// Calculate the total value of the transaction inputs
	let input_total: Amount = decoded
		.vin
		.iter()
		.filter_map(|input| {
			// Get the transaction output for this input
			// Find the output corresponding to this input index
			decoded.vout
				.iter()
				.find(|out| out.n == input.vout.unwrap())
				.map(|out| out.value)
		})
		.sum();
	
	// Calculate the total fees for the transaction
	let fee = 0;

	// Ok(format!("decoded: {:?}", decoded))

	Ok(format!("address = {}, amount = {}, fee = {}", address, amount, fee))
}

// #[tauri::command]
// //for testing only
// async fn init_test() -> String {
//     let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
//     //TODO: Create this in start_bitcoind and conversly set it to none if we close it.
//     let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");
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
//     let desc = build_high_descriptor(&Client, &keys).unwrap();

//     format!("testing {} {}", desc, desc.sanity_check().unwrap() == ())
// }



fn main() {
    let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    //TODO: Create this in start_bitcoind and conversly set it to none if we close it.
    let Client = bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth).expect("could not connect to bitcoin core");

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
        install_sd_deps,
        refresh_cd,
        distribute_shards_sd2,
        distribute_shards_sd3,
        distribute_shards_sd4,
        distribute_shards_sd5,
        distribute_shards_sd6,
        distribute_shards_sd7,
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
		sign_psbt,
		finalize_psbt,
		broadcast_tx,
		decode_raw_tx,
        ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
