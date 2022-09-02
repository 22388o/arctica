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


struct MyState(Mutex<Result<RpcBlockchain, bdk::Error>>);


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
fn create_bootable_usb() -> String {
	println!("run a rust test");
	println!("run a shell command");
	let output = Command::new("bash")
            .args(["./scripts/test.sh"])
            .output()
            .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    }
    println!(";");

	format!("completed with no problems")
	//"printf '%s\n' n y g y | mksub ~/arctica/resources/ubunntu-22.04-desktop-amd64.iso"
	//"kvm -m 2048 -hdb /dev/sda -boot d -cdrom ~/arctica/resources/ubuntu-22.04-deskotp-amd64.iso"
  	// println!("I was invoked from JS, with this message: {}, {}", invoke_message, height);
}

#[tauri::command]
fn create_bootable_usb_test() -> String {
	println!("run a rust command");
	println!("run a shell command");
	let output = Command::new("bash")
            .args(["./scripts/test.sh"])
            .output()
            .expect("failed to execute process");
    for byte in output.stdout {
    	print!("{}", byte as char);
    }
    println!(";");
	format!("completed with no problems")
	//"printf '%s\n' n y g y | mksub ~/arctica/resources/ubunntu-22.04-desktop-amd64.iso"
	//"kvm -m 2048 -hdb /dev/sda -boot d -cdrom ~/arctica/resources/ubuntu-22.04-deskotp-amd64.iso"
  	// println!("I was invoked from JS, with this message: {}, {}", invoke_message, height);
}

#[tauri::command]
fn print_rust(data: &str) -> String {
	println!("run a rust command and accept input");
	println!("input = {}", data);
	format!("completed with no problems")
	//"printf '%s\n' n y g y | mksub ~/arctica/resources/ubunntu-22.04-desktop-amd64.iso"
	//"kvm -m 2048 -hdb /dev/sda -boot d -cdrom ~/arctica/resources/ubuntu-22.04-deskotp-amd64.iso"
  	// println!("I was invoked from JS, with this message: {}, {}", invoke_message, height);
}




fn main() {
  	tauri::Builder::default()
  	.manage(MyState(Mutex::new(getblockchain())))
  	.invoke_handler(tauri::generate_handler![create_bootable_usb, create_bootable_usb_test])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
