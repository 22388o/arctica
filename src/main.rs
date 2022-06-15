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
fn my_custom_command(invoke_message: String, state: tauri::State<MyState>) {
	let blockchain = match &state.0.lock().unwrap().deref() {
		Ok(blockchain) => blockchain,
		Err(e) => panic!("faild blockcahin: {}", e)
	};
	// let height = match blockchain.get_height() {
 //        Ok(height)  => height,
 //        Err(e) => panic!("Error getting height: {}", e)
 //    };
  	// println!("I was invoked from JS, with this message: {}, {}", invoke_message, height);
}

    // println!("Address #0: {}", wallet.get_address(New)?);
    // println!("Address #1: {}", wallet.get_address(New)?);
    // println!("Address #2: {}", wallet.get_address(New)?);

fn main() {
  	tauri::Builder::default()
  	.manage(MyState(Mutex::new(getblockchain())))
  	.invoke_handler(tauri::generate_handler![my_custom_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
