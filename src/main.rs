#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use bdk::{Wallet};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bitcoincore_rpc::Client;
use bdk::blockchain::rpc::Auth;
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


struct MyState(Mutex<Result<RpcBlockchain, bdk::Error>>);

fn write(name: String, value:String) {
		let config_file = "config.txt";
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

    let mut file = File::create(&config_file).expect("Colud not Open file");
    file.write_all(newfile.as_bytes()).expect("Could not rewrite file");
}


fn read() {
		let config_file = "config.txt";

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
            println!("line: {}={}", n, v);
        }
    }
}


#[tauri::command]
fn getblockchain() -> Result<RpcBlockchain, bdk::Error>{
	let user_pass: Auth = Auth::UserPass{username: "rpcuser".to_string(), password: "477028".to_string()};
    let config: RpcConfig = RpcConfig {
	    url: "127.0.0.1:8332".to_string(),
	    auth: user_pass,
	    network: bdk::bitcoin::Network::Bitcoin,
	    wallet_name: "wallet_name".to_string(),
	    skip_blocks: None,
	};
	let blockchain = RpcBlockchain::from_config(&config);
    return blockchain
}

#[tauri::command]
fn test_function() -> String {
	println!("this is a test");
	let output = Command::new("echo")
            .args(["the test worked"])
            .output()
            .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    }
    println!(";");
	format!("completed with no problems")
}


fn mount_sd() -> String {
	println!("mounting the current SD");
	let output = Command::new("bash")
            .args(["./scripts/mount-sd.sh"])
            .output()
            .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    }
    println!(";");
	format!("completed with no problems")
}

fn create_config() -> String {
	println!("creating the config file");
	let output = Command::new("bash")
            .args(["./scripts/create-config.sh"])
            .output()
            .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    }
    println!(";");
	format!("completed with no problems")
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

#[tauri::command]
async fn install_kvm() -> String {
	println!("installing KVM & dependencies");
	let output = Command::new("bash")
		.args(["./scripts/install-kvm.sh"])
		.output()
		.expect("failed to execute process");
	for byte in output.stdout{
		print!("{}", byte as char);
	}
	println!(";");

	format!("completed with no problems")
}

//front-end: setup 1
//create the bitcoin dotfile on the local machine internal disk, where block data will be stored 
//this currently requires the use of sudo, else I can't break into the home dir, not ideal, revise if possible
#[tauri::command]
async fn make_bitcoin_dotfile() -> String {
	println!("Making Bitcoin dotfile");
	let output = Command::new("bash")
		.args(["./scripts/makebitcoindotfile.sh"])
    .output()
    .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    };
    println!(";");
	format!("completed with no problems")
}


//this will be the initial flash of all 7 SD cards
//runs on setup 4-10
#[tauri::command]
async fn create_bootable_usb(number:  &str, setup: &str) -> Result<String, String> {
	println!("creating bootable ubuntu device = {} {}", number, setup);
	// let output = Command::new("bash")
 //        .args(["./scripts/clone-sd.sh"])
 //        .output()
 //        .expect("failed to execute process");
 //    for byte in output.stdout {
 //    	print!("{}", byte as char);
 //    }
  print_rust("testdata");
//   create_config();
//   mount_sd();
   write(number.to_string(), "true".to_string());
//   write(setup.to_string(), "true".to_string());
  println!(";");
	Ok(format!("completed with no problems"))
}


#[tauri::command]
fn print_rust(data: &str) -> String {
	println!("input = {}", data);
	format!("completed with no problems")
}




fn main() {
  	tauri::Builder::default()
  	.manage(MyState(Mutex::new(getblockchain())))
  	.invoke_handler(tauri::generate_handler![test_function, print_rust, create_bootable_usb, make_bitcoin_dotfile, obtain_ubuntu, install_kvm])
  	//.invoke_handler(tauri::generate_handler![])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}