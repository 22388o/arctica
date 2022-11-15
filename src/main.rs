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
use home::home_dir;
use secp256k1::{rand, Secp256k1, SecretKey};
use tauri::State;



struct TauriState(Mutex<RpcConfig>, Mutex<Option<Wallet<MemoryDatabase>>>, Mutex<Option<Wallet<MemoryDatabase>>>, Mutex<Option<Wallet<MemoryDatabase>>>);

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

#[tauri::command]
fn generate_key() -> Result<bitcoin::PrivateKey, bitcoincore_rpc::Error> {
	let secp = Secp256k1::new();
	let secret_key = SecretKey::new(&mut rand::thread_rng());
	Ok(bitcoin::PrivateKey::new(secret_key, bitcoin::Network::Bitcoin))
}

fn build_high_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
	let mut keys = Vec::new();
	for i in 0..11 {
		keys.push(generate_key().expect("could not get key"))
	}
	let four_years = blockchain.get_height().unwrap()+210379;
	let month = 4382;
	Ok(format!("and(thresh(5,after({}),after({}),after({}),after({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({})),thresh(2,after({}),after({}),pk({}),pk({}),pk({}),pk({})))", four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years, keys[7], keys[8], keys[9], keys[10]))
}

fn build_med_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
	let mut keys = Vec::new();
	for i in 0..11 {
		keys.push(generate_key().expect("could not get key"))
	}
	let four_years = blockchain.get_height().unwrap()+210379;
	let month = 4382;
	Ok(format!("and(thresh(5,after({}),after({}),after({}),after({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({})),thresh(2,after({}),after({}),pk({}),pk({}),pk({}),pk({})))", four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years, keys[7], keys[8], keys[9], keys[10]))
}


fn build_low_descriptor(blockchain: &RpcBlockchain) -> Result<String, bdk::Error> {
	let mut keys = Vec::new();
	for i in 0..11 {
		keys.push(generate_key().expect("could not get key"))
	}
	let four_years = blockchain.get_height().unwrap()+210379;
	let month = 4382;
	Ok(format!("and(thresh(5,after({}),after({}),after({}),after({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({}),pk({})),thresh(2,after({}),after({}),pk({}),pk({}),pk({}),pk({})))", four_years, four_years+(month), four_years+(month*2), four_years+(month*3), keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years, four_years, keys[7], keys[8], keys[9], keys[10]))
}



#[tauri::command]
fn generate_wallet(state: State<TauriState>) -> Result<(), bdk::Error> {
	//todo get block chain via the state
	let blockchain = RpcBlockchain::from_config(&*state.0.lock().unwrap())?;
	let high_desc = build_high_descriptor(&blockchain)?;
	let med_desc = build_med_descriptor(&blockchain)?;
	let low_desc = build_low_descriptor(&blockchain)?;
	*state.1.lock().unwrap() = Some(Wallet::new(&high_desc, None, bitcoin::Network::Bitcoin, MemoryDatabase::default())?);
	*state.2.lock().unwrap() = Some(Wallet::new(&med_desc, None, bitcoin::Network::Bitcoin, MemoryDatabase::default())?);
	*state.3.lock().unwrap() = Some(Wallet::new(&low_desc, None, bitcoin::Network::Bitcoin, MemoryDatabase::default())?);
	return Ok(())
}


#[tauri::command]
async fn test_function() -> String {
	println!("this is a test");
	let output = Command::new("echo")
            .args(["the test worked"])
            .output()
            .expect("failed to execute process");
    // for byte in output.stdout {
    // 	print!("{}", byte as char);
    // }
    println!(";");
	format!("{:?}", output)
}


//front-end: boot
// runs on the boot screen when user clicks install, downloads latest copy of tails
#[tauri::command]
async fn obtain_ubuntu() -> String {
	println!("obtaining & creating modified ubuntu iso");
	let output = Command::new("bash")
           .args(["./scripts/init-iso.sh"])
           .output()
           .expect("failed to execute process");
   for byte in output.stdout {
   	print!("{}", byte as char);
   }
    println!(";");

	format!("completed with no problems")
}

//this will be the initial flash of all 7 SD cards
//runs on setup 4-10
#[tauri::command]
async fn create_bootable_usb(number:  &str, setup: &str) -> Result<String, String> {
    write("type".to_string(), "sdcard".to_string());
    write("sdNumber".to_string(), number.to_string());
    write("setupStep".to_string(), setup.to_string());
	println!("creating bootable ubuntu device = {} {}", number, setup);
	let output = Command::new("bash")
        .args(["./scripts/clone-sd.sh"])
        .output()
        .expect("failed to execute process");
    for byte in output.stdout {
        print!("{}", byte as char);
    }
  println!(";");
        Ok(format!("Completed with no problems"))

    //use this if you want to pass a response of the stderr to front end
	// Ok(format!("{:?}", output))
}

#[tauri::command]
async fn create_setup_cd() -> String {
    write("type".to_string(), "setupcd".to_string());
	println!("creating setup CD");
	let output = Command::new("bash")
        .args(["/home/ubuntu/scripts/create-setup-cd.sh"])
        .output()
        .expect("failed to execute process");
  println!(";");
	format!("{:?}", output)
}

#[tauri::command]
async fn copy_setup_cd() -> String {
	println!("copy setup CD to ramdisk");
	let output = Command::new("bash")
        .args(["/home/ubuntu/scripts/copy-setup-cd.sh"])
        .output()
        .expect("failed to execute process");
  println!(";");
	format!("{:?}", output)
}

#[tauri::command]
async fn packup() -> String {
	println!("packing up sensitive info");
	let output = Command::new("bash")
        .args(["/home/ubuntu/scripts/packup.sh"])
        .output()
        .expect("failed to execute process");
  println!(";");
	format!("{:?}", output)
}

#[tauri::command]
async fn unpack() -> String {
	println!("unpacking sensitive info");
	let output = Command::new("bash")
        .args(["/home/ubuntu/scripts/unpack.sh"])
        .output()
        .expect("failed to execute process");
  println!(";");
	format!("{:?}", output)
}

#[tauri::command]
async fn create_ramdisk() -> String {
	println!("creating ramdisk");
	let output = Command::new("bash")
        .args(["/home/ubuntu/scripts/create-ramdisk.sh"])
        .output()
        .expect("failed to execute process");
  println!(";");
	format!("{:?}", output)
}

#[tauri::command]
fn read_setup_cd() -> std::string::String {
    let config_file = "/media/ubuntu/CDROM/config.txt";
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
fn print_rust(data: &str) -> String {
	println!("input = {}", data);
	format!("completed with no problems")
}


#[tauri::command]
async fn create_wallet() -> String {
	println!("creating simulated bitcoin wallet");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/create-wallet.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn combine_shards() -> String {
	println!("combining shards in /mnt/ramdisk/shards");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/combine-shards.sh"])
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
		.args(["/home/ubuntu/scripts/mount-internal.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn install_sd_deps() -> String {
	println!("installing deps required by SD card");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/install-sd-deps.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn refresh_setup_cd() -> String {
	println!("refreshing setupCD with latest data");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/refresh-setup-cd.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn distribute_2_shards() -> String {
	println!("distributing 2 privacy key shards to the current SD card");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/distribute-2-shards.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn distribute_1_shard() -> String {
	println!("distributing 1 privacy key shard to the current SD card");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/distribute-1-shard.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn create_descriptor() -> String {
	println!("creating descriptor from 7 xpubs");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/create-descriptor.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn copy_descriptor() -> String {
	println!("copying descriptor from setupCD dump to sensitive dir");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/copy-descriptor.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn extract_masterkey() -> String {
	println!("extracting masterkey from setupCD dump");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/extract-masterkey.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn create_backup() -> String {
	println!("creating backup directory of the current SD");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/create-backup.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn make_backup() -> String {
	println!("making backup iso of the current SD and burning to CD");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/make-backup.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

#[tauri::command]
async fn start_bitcoind() -> String {
	println!("starting the bitcoin daemon");
	let output = Command::new("bash")
		.args(["/home/ubuntu/scripts/start-bitcoind.sh"])
		.output()
		.expect("failed to execute process");
	format!("{:?}", output)
}

fn main() {
	let user_pass: bdk::blockchain::rpc::Auth = bdk::blockchain::rpc::Auth::UserPass{username: "rpcuser".to_string(), password: "477028".to_string()};
    let config: RpcConfig = RpcConfig {
	    url: "127.0.0.1:8332".to_string(),
	    auth: user_pass,
	    network: bdk::bitcoin::Network::Bitcoin,
	    wallet_name: "wallet_name".to_string(),
	    sync_params: None,
	};
  	tauri::Builder::default()
  	.manage(TauriState(Mutex::new(config), Mutex::new(None), Mutex::new(None), Mutex::new(None)))
  	.invoke_handler(tauri::generate_handler![
        test_function,
         print_rust,
          create_wallet,
           create_bootable_usb,
            create_setup_cd,
             read_setup_cd,
             copy_setup_cd,
              obtain_ubuntu,
               async_write,
                read,
                 combine_shards,
                  mount_internal,
                   create_ramdisk,
                    packup,
                     unpack,
                     install_sd_deps,
                     refresh_setup_cd,
                     distribute_2_shards,
                     distribute_1_shard,
                     create_descriptor,
                     copy_descriptor,
                     extract_masterkey,
                     create_backup,
                     make_backup,
                     start_bitcoind,
                     ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}