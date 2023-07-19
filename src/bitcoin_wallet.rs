use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::bitcoincore_rpc_json::{AddressType, ImportDescriptors};
use bitcoincore_rpc::bitcoincore_rpc_json::{WalletProcessPsbtResult, ListTransactionResult, Bip125Replaceable, GetTransactionResultDetailCategory, WalletCreateFundedPsbtResult};
use bitcoincore_rpc::bitcoin::Address;
use bitcoincore_rpc::bitcoin::Network;
use bitcoincore_rpc::bitcoin::Script;
use bitcoin;
use bitcoin::consensus::serialize;
use bitcoin::consensus::deserialize;
use bitcoin::psbt::PartiallySignedTransaction;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::{time::Duration};
use std::process::Stdio;
use std::collections::{HashMap, HashSet};
use serde_json::{json};
use serde::{Serialize, Deserialize};
extern crate hex;


//import functions from helper
use crate::helper::{
	get_user, get_home, is_dir_empty, get_uuid, store_psbt, get_descriptor_checksum, retrieve_decay_time, 
	retrieve_decay_time_integer, unix_to_block_height, store_unsigned_psbt
};

//custom structs
#[derive(Clone, Serialize)]
struct CustomTransaction {
	id: i32,
    info: CustomWalletTxInfo,
    detail: CustomGetTransactionResultDetail,
    trusted: Option<bool>,
    comment: Option<String>,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
struct CustomGetTransactionResultDetail {
    address: Option<String>,
    category: String,
    amount: i64,
    label: Option<String>,
    vout: u32,
    fee: Option<i64>,
    abandoned: Option<bool>,
}

impl PartialEq for CustomTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.info.txid == other.info.txid &&
        self.detail.address == other.detail.address &&
        self.detail.amount == other.detail.amount
    }
}



//functions library

//RPC command
// ./bitcoin-cli createwallet "wallet name" ___, true, ___, ____
//creates a blank wallet
pub fn create_wallet(wallet: String, hwnumber: &String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//create blank wallet
	let output = match client.create_wallet(&(wallet.to_string()+"_wallet"+&hwnumber.to_string()), None, Some(true), None, None) {
		Ok(file) => file,
		Err(err) => return Err(err.to_string()),
	};
	Ok(format!("SUCCESS creating the wallet {:?}", output))
}

//builds the high security descriptor, 7 of 11 thresh with decay. 4 of the 11 keys will go to the BPS
pub fn build_high_descriptor(keys: &Vec<String>, hwnumber: &String, internal: bool) -> Result<String, String> {
	println!("calculating 4 year block time span");
	//decay1 which is the timelock var
	let four_years_int = retrieve_decay_time_integer("delayed_decay1".to_string()); 
	let four_years = four_years_int.to_string();
	println!("delayed wallet decay1 threshold: {}", four_years);
	//decay2 which is the first threshold decay
	let four_years_two_months_int = retrieve_decay_time_integer("delayed_decay2".to_string()); 
	let four_years_two_months = four_years_two_months_int.to_string();
	println!("delayed wallet decay2 threshold: {}", four_years_two_months);
	//decay3 which is the second threshold decay
	let four_years_four_months_int = retrieve_decay_time_integer("delayed_decay3".to_string()); 
	let four_years_four_months = four_years_four_months_int.to_string();
	println!("delayed wallet decay3 threshold: {}", four_years_four_months);
	//decay4 which is the third threshold decay
	let four_years_six_months_int = retrieve_decay_time_integer("delayed_decay4".to_string()); 
	let four_years_six_months = four_years_six_months_int.to_string();
	println!("delayed wallet decay4 threshold: {}", four_years_six_months);
	//decay5 which is the fourth threshold decay
	let four_years_eight_months_int = retrieve_decay_time_integer("delayed_decay5".to_string()); 
	let four_years_eight_months = four_years_eight_months_int.to_string();
	println!("delayed wallet decay5 threshold: {}", four_years_eight_months);
	println!("reading xpriv");
	//read xpriv from file to string
	let mut private_key = "private_key";
	//internal change condition is true
	if internal == true {
		private_key = "private_change_key";
	}
	let xpriv = match fs::read_to_string(&("/mnt/ramdisk/sensitive/".to_string()+&(private_key.to_string())+&(hwnumber.to_string()))){
		Ok(xpriv)=> xpriv,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	println!("{}", xpriv);
	//determine how to format the descriptor based on which HW the user is currently using
	if hwnumber == "1"{
		println!("Found HW = 1");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output: String = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "2"{
		println!("Found HW = 2");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "3"{
		println!("Found HW = 3");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "4"{
		println!("Found HW = 4");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "5"{
		println!("Found HW = 5");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "6"{
		println!("Found HW = 6");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], xpriv, keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "7"{
		println!("Found HW = 7");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], xpriv, four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "timemachine1"{
		println!("Found HW = timemachine1");
		let timemachinexpriv = match fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_private_key".to_string()+&(hwnumber.to_string()))){
			Ok(xpriv)=> xpriv,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, timemachinexpriv, keys[8], keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if hwnumber == "timemachine2"{
		println!("Found HW = timemachine2");
		let timemachinexpriv = match fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_".to_string()+&(private_key.to_string())+&(hwnumber.to_string()))){
			Ok(xpriv)=> xpriv,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], timemachinexpriv, keys[9], keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if hwnumber == "timemachine3"{
		println!("Found HW = timemachine3");
		let timemachinexpriv = match fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_".to_string()+&(private_key.to_string())+&(hwnumber.to_string()))){
			Ok(xpriv)=> xpriv,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], timemachinexpriv, keys[10], four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else if hwnumber == "timemachine4"{
		println!("Found HW = timemachine4");
		let timemachinexpriv = match fs::read_to_string(&("/mnt/ramdisk/CDROM/timemachinekeys/time_machine_".to_string()+&(private_key.to_string())+&(hwnumber.to_string()))){
			Ok(xpriv)=> xpriv,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], timemachinexpriv, four_years, four_years);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}else{
		println!("no valid hwnumber param found, creating read only desc");
		let descriptor = format!("wsh(and_v(v:thresh(5,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}),snu:after({}),snu:after({})),thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({}),snu:after({}))))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_two_months, four_years_four_months, four_years_six_months, four_years_eight_months, keys[7], keys[8], keys[9], keys[10], four_years, four_years);
		println!("Read only DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))	
	}

}

//builds the medium security descriptor, 2 of 7 thresh with decay. 
pub fn build_med_descriptor(keys: &Vec<String>, hwnumber: &String, internal: bool) -> Result<String, String> {
	println!("calculating 4 year block time span");
	//four_years_eight_months is a unix timestamp created with create_setup_cd
    let four_years_eight_months_int = retrieve_decay_time_integer("immediate_decay".to_string()); 
	let four_years_eight_months = four_years_eight_months_int.to_string();
	println!("immediate wallet decay threshold: {}", four_years_eight_months);

	println!("reading xpriv");
	let mut private_key = "private_key";
	//internal change condition is true
	if internal == true {
		private_key = "private_change_key";
	}
	let xpriv = match fs::read_to_string(&("/mnt/ramdisk/sensitive/".to_string()+&(private_key.to_string())+&(hwnumber.to_string()))){
		Ok(xpriv)=> xpriv,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	println!("{}", xpriv);
	//determine how to format the descriptor based on which HW the user is currently using
	if hwnumber == "1"{
		println!("Found HW = 1");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "2"{
		println!("Found HW = 2");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "3"{
		println!("Found HW = 3");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "4"{
		println!("Found HW = 4");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "5"{
		println!("Found HW = 5");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "6"{
		println!("Found HW = 6");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], xpriv, keys[3], keys[4], xpriv, keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else if hwnumber == "7"{
		println!("Found HW = 7");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], xpriv, four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}else{
		println!("no valid hwnumber param found, creating read only desc");
		let descriptor = format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),s:pk({}),snu:after({})))", keys[0], keys[1], keys[2], keys[3], keys[4], keys[5], keys[6], four_years_eight_months);
		println!("DESC: {}", descriptor);
		let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
	}
}

//builds the low security descriptor, 1 of 7 thresh, used for tripwire
//TODO this needs to use its own special keypair or it will be a privacy leak once implemented
//TODO this may not need child key designators /* because it seems to use hardened keys but have not tested this descriptor yet
	pub fn build_low_descriptor(keys: &Vec<String>, hwnumber: &String, internal: bool) -> Result<String, String> {
		println!("reading xpriv");
		let mut private_key = "private_key";
		//internal change condition is true, use private_change_key instead
		if internal == true {
			private_key = "private_change_key";
		}
		let xpriv = match fs::read_to_string(&("/mnt/ramdisk/sensitive/".to_string()+&(private_key)+&(hwnumber.to_string()))){
			Ok(xpriv)=> xpriv,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		println!("{}", xpriv);
		//determine how to format the descriptor based on which HW the user is currently using
		if hwnumber == "1"{
			println!("Found HW = 1");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", xpriv, keys[1], keys[2], keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "2"{
			println!("Found HW = 2");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], xpriv, keys[2], keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "3"{
			println!("Found HW = 3");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], xpriv, keys[3], keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "4"{
			println!("Found HW = 4");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], xpriv, keys[4], keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "5"{
			println!("Found HW = 5");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], xpriv, keys[5], keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "6"{
			println!("Found HW = 6");
			let descriptor = format!("wsh(c:or_i(pk_k({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),or_i(pk_h({}),pk_h({}))))))))", keys[0], keys[1], keys[2], keys[3], keys[4], xpriv, keys[6]);
			println!("DESC: {}", descriptor);
			let output = get_descriptor_checksum(descriptor);
		Ok(format!("{}", output))
		}else if hwnumber == "7"{
			println!("Found HW = 7");
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

//RPC command
// ./bitcoin-cli -rpcwallet=<filepath>|"wallet_name" importdescriptors "requests"
//requests is a JSON and is formatted as follows
//'[{"desc": "<descriptor goes here>", "active":true, "range":[0,100], "next_index":0, "timestamp": <start_time_timestamp>}]'
//acceptable params here are "low" & "low_change", "immediate" & "immediate_change", "delayed" & "delayed_change"; hwNumber 1-7; internal: true designates change descriptor
//TODO timestamp is not currently fucntional due to a type mismatch, timestamp within the ImportDescriptors struct wants bitcoin::timelock:time
pub fn import_descriptor(wallet: String, hwnumber: &String, internal: bool) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(wallet.to_string())+"_wallet"+ &(hwnumber.to_string())), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	let mut descriptor_str = "_descriptor";
	if internal == true {
		descriptor_str = "_change_descriptor"
	}
	//read the descriptor to a string from file
		let desc: String = match fs::read_to_string(&("/mnt/ramdisk/sensitive/descriptors/".to_string()+&(wallet.to_string())+&(descriptor_str.to_string()) + &(hwnumber.to_string()))){
			Ok(desc)=> desc,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};

	//obtain the start time from file
	let start_time = retrieve_decay_time("start_time".to_string());
	let mut change = Some(true);
	if internal == false {
		change = Some(false);
	}
	//import the descriptors into the wallet file
	let output = match client.import_descriptors(ImportDescriptors {
		descriptor: desc,
		timestamp: start_time,
		active: Some(true),
		range: Some((0, 100)),
		next_index: Some(0),
		internal: change,
		label: None
	}){
			Ok(file) => file,
			Err(err) => return Err(err.to_string()),
		
	};
	Ok(format!("Success in importing descriptor...{:?}", output))
}

#[tauri::command]
//get a new address
//accepts "low", "immediate", and "delayed" as a param
//equivalent to... Command::new("./bitcoin-25.0/bin/bitcoin-cli").args([&("-rpcwallet=".to_string()+&(wallet.to_string())+"_wallet"), "getnewaddress"])
//must be done with client url param URL=<hostname>/wallet/<wallet_name>
pub async fn get_address(walletname: String, hwnumber:String) -> Result<String, String> {
	// //need to kill eog here if it's running as it will show a stale QR and/or crash otherwise
	// let pidof = Command::new("pidof").arg("eog").output().unwrap();
	// let pid = std::str::from_utf8(&pidof.stdout).unwrap().trim();
	// //kill pid
	// Command::new("kill").args(["-9", &pid]).output().unwrap();
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//address labels can be added here
	let address_type = Some(AddressType::Bech32);
	let address = match client.get_new_address(None, address_type){
		Ok(addr) => addr,
		Err(err) => return Ok(format!("{}", err.to_string()))
	};
	// //create a QR code for the address
	let address_str = address.to_string();
	//delete stale QR file
	Command::new("sudo").args(["rm", "/mnt/ramdisk/qrcode.png"]).output().unwrap();
	//file destination for QR code
	let mut file = match File::create("/mnt/ramdisk/qrcode.svg"){
		Ok(file) => file,
		Err(err) => return Ok(format!("{}", err.to_string()))
	};
	//create QR code
	let output = Command::new("qrencode").args(["-s", "6", "-l", "H", "-o", "/mnt/ramdisk/qrcode.png", &address_str]).output().unwrap();
	if !output.status.success() {
	return Ok(format!("ERROR in generating QR code {}", std::str::from_utf8(&output.stderr).unwrap()));
	}
	Ok(format!("{}", address))
}

#[tauri::command]
//calculate the current balance of the tripwire wallet
pub async fn get_balance(walletname:String, hwnumber:String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//get wallet balance
	match client.get_balance(None, Some(true)){
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

#[tauri::command]
//retrieve the current transaction history for the immediate wallet
pub async fn get_transactions(walletname: String, hwnumber:String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
   let transactions: Vec<ListTransactionResult> = match client.list_transactions(None, None, None, Some(true)) {
	Ok(tx) => tx,
	Err(err) => return Ok(format!("{}", err.to_string()))
   };
   //handler for empy wallet with no transaction history
   if transactions.is_empty() {
	return Ok(format!("empty123321"))
   }
   else{
	let mut custom_transactions: Vec<CustomTransaction> = Vec::new();
	let mut x = 0;
    //append result to a custom tx struct
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
			custom_transactions.push(custom_tx);
			x += 1;
		
	}

	    // Group transactions into a hashmap by their transaction ID
		// let mut tx_groups: HashMap<&str, Vec<CustomTransaction>> = HashMap::new();
		// // let mut txids = Vec::new();
		// for tx in custom_transactions {
		// 	let custom_tx = {
		// 		txid: tx.info.txid.to_string(),
		// 	};

		// 	//add the txid to the hashmap
		// 	let txid = tx.info.txid.to_string();
		// 	let tx_group = tx_groups.entry(&txid).or_insert(vec![]);
		// 	tx_group.push(custom_tx);
		// }
		//iterate through each group of transactions with the same txid
		//if any txs are found in a batch of txids to have indentical address and amount fields, exclude them from the results

	

		// let txids: Vec<String> = custom_transactions.iter().map(|tx| tx.info.txid.to_string()).collect();
	//check for duplicate txids. 
	//if a batch of txids has >2 outputs & atleast two duplicate amounts & addresses...assume change and filter from results
	let json_string = serde_json::to_string(&custom_transactions).unwrap();
	println!("{}", json_string);
	Ok(format!("{}", json_string))
   }
}

#[tauri::command]
//generate a PSBT for the immediate wallet
//will require additional logic to spend when under decay threshold
//currently only generates a PSBT for Key 1 and Key 2, which are HW 1 and HW 2 respectively
pub async fn generate_psbt(walletname: String, hwnumber:String, recipient: &str, amount: f64, fee: u64) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//create the directory where the PSBT will live if it does not exist
   let a = std::path::Path::new("/mnt/ramdisk/psbt").exists();
   if a == false{
       //make psbt dir
       let output = Command::new("mkdir").args(["/mnt/ramdisk/psbt"]).output().unwrap();
       if !output.status.success() {
       return Ok(format!("ERROR in creating /mnt/ramdisk/psbt dir {}", std::str::from_utf8(&output.stderr).unwrap()));
       }
   }
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
// 	&inputs, //no inputs specified
//   let psbt_result = client.wallet_create_funded_psbt(
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

//create the input JSON
let json_input = json!([]);
//creat the output JSON
let json_output = json!([{
	recipient: amount
}]);
//empty options JSON
let mut options = json!({
});
//if the user specifies a custom fee, append it to the options JSON
if fee != 0{
	options["fee_rate"] = json!(fee);
}


// let locktime_output = Command::new("date").args(["+%s"]).output().unwrap();
let locktime = retrieve_median_blocktime();
//1st attempt
let psbt_output1 = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli"))
.args([&("-rpcwallet=".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), 
"walletcreatefundedpsbt", 
&json_input.to_string(), //empty array lets core pick the inputs
&json_output.to_string(), //receive address & output amount
&locktime, //current unix time
&options.to_string() //manually providing fee rate if applicable
]) 
.output()
.unwrap();
//if insufficient funds error, attempt to subtract fees from outputs
let fee_check = std::str::from_utf8(&psbt_output1.stderr).unwrap();
if fee_check.contains("Insufficient funds"){
	options["subtractFeeFromOutputs"] = json!([]);
};
//2nd attempt
let psbt_output2 = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli"))
.args([&("-rpcwallet=".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), 
"walletcreatefundedpsbt", 
&json_input.to_string(), //unchanged
&json_output.to_string(), //unchanged
&locktime, //unchanged
&options.to_string() //enable subtractFeeFromOutputs option
]) 
.output()
.unwrap();
//handle error if failed
if !psbt_output2.status.success() {
	return Ok(format!("ERROR in generating PSBT = {}", std::str::from_utf8(&psbt_output2.stderr).unwrap()));
}

//convert psbt to string from hex
let psbt_str = String::from_utf8(psbt_output2.stdout).unwrap();
//convert psbt string to an rpc crate struct
let psbt: WalletCreateFundedPsbtResult = match serde_json::from_str(&psbt_str) {
	Ok(psbt)=> psbt,
	Err(err)=> return Ok(format!("{}", err.to_string()))
};
//declare the destination for the PSBT file
let file_dest = "/mnt/ramdisk/psbt/psbt".to_string();
//store the transaction as a file
match store_unsigned_psbt(&psbt, file_dest) {
	Ok(_) => {},
	Err(err) => return Err("ERROR could not store PSBT: ".to_string()+&err)
	};
Ok(format!("PSBT: {:?}", psbt))
}

//start bitcoin core daemon
#[tauri::command]
pub async fn start_bitcoind(reindex:bool, networkactive:bool) -> String {
	//obtain the UUID of the internal storage disk
	let uuid = get_uuid();
	//open file permissions of .bitcoin for settings.tmp
	//the only time this  block should be required is immediately following initial setup
	let output = Command::new("sudo").args(["chmod", "777", &(get_home().to_string()+&"/.bitcoin".to_string())]).output().unwrap();
	if !output.status.success() {
		return format!("ERROR opening .bitcoin permissions = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//starting bitcoind with the network active...this will begin syncing the blockchain
	if networkactive == true{
		//enable networking 
		let output = Command::new("sudo").args(["nmcli", "networking", "on"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR disabling networking = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//handle error condition where the UUID parse failed
		if uuid == "ERROR in parsing /media/user" {
			return format!("Error in parsing /media/user to get uuid");
		}
		//failure condition, internal drive not properly mounted
		else if uuid == "none"{
			return format!("ERROR could not find a valid UUID in /media/$user");
		}else{
			//check if walletdir exists and if not create it
			let a = std::path::Path::new("/mnt/ramdisk/sensitive/wallets").exists();
			if a == false {
				let output = Command::new("mkdir").args(["/mnt/ramdisk/sensitive/wallets"]).output().unwrap();
				if !output.status.success() {
					return format!("ERROR in starting bitcoin daemon with creating ../sensitive/wallets dir = {}", std::str::from_utf8(&output.stderr).unwrap());
				}
			}
		}
	//starting bitcoind with the network inactive...this will not sync the blockchain
	} else if networkactive == false{
		//disable networking
		//networing is force disabled for key generation on all Hardware Wallets and should persist across restarts. 
		let output = Command::new("sudo").args(["nmcli", "networking", "off"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR disabling networking = {}", std::str::from_utf8(&output.stderr).unwrap());
		}
		//check if walletdir exists and if not create it
		let a = std::path::Path::new("/mnt/ramdisk/sensitive/wallets").exists();
		if a == false {
			let output = Command::new("mkdir").args(["/mnt/ramdisk/sensitive/wallets"]).output().unwrap();
			if !output.status.success() {
				return format!("ERROR in starting bitcoin daemon with creating ../sensitive/wallets dir = {}", std::str::from_utf8(&output.stderr).unwrap());
			}
		}
	}
	//next we will start the bitcoin daemon with proper params
	if networkactive == false{
		//this is run when we desire to disable networking
		//this will prevent block sync
		//use this function when starting core daemon on any offline device


		// std::thread::spawn( ||{
		// 	Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoind"))
		// 	.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), "-walletdir=/mnt/ramdisk/sensitive/wallets", "-networkactive=0"])
		// 	.stdout(Stdio::null())
		// 	.stderr(Stdio::null())
		// 	.stdin(Stdio::null())
		// 	.spawn();
		// 	});
		
		Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoind"))
		.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), "-walletdir=/mnt/ramdisk/sensitive/wallets", "-networkactive=0"])
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.stdin(Stdio::null())
		.spawn();

		return format!("SUCCESS in starting bitcoin daemon with networking disabled");

	}
	else if reindex == true {
		//this is run when we desire to reindex the blocksdb
		std::thread::spawn( ||{
			//redeclare dynamic vars within the new scope
			let uuid = get_uuid();
			let host = Command::new(&("ls")).args([&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home")]).output().unwrap();
			let host_user = std::str::from_utf8(&host.stdout).unwrap().trim();
			//spawn as a child process on a seperate thread, nullify the output
			//note there is a -reindex flag here for this conditional
			Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoind"))
			.args(["-reindex", "-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), &("-datadir=/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin"), "-walletdir=/mnt/ramdisk/sensitive/wallets"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.stdin(Stdio::null())
			.spawn();
			});
	}	else {
		//this should be the normal way to start bitcoind
		std::thread::spawn( ||{
			//redeclare dynamic vars within the new scope
			let uuid = get_uuid();
			let host = Command::new(&("ls")).args([&("/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home")]).output().unwrap();
			let host_user = std::str::from_utf8(&host.stdout).unwrap().trim();
			//spawn as a child process on a seperate thread, nullify the output
			Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoind"))
			.args(["-debuglogfile=/mnt/ramdisk/debug.log", &("-conf=".to_string()+&get_home()+"/.bitcoin/bitcoin.conf"), &("-datadir=/media/".to_string()+&get_user()+"/"+&(uuid.to_string())+"/home/"+&(host_user.to_string())+"/.bitcoin"), "-walletdir=/mnt/ramdisk/sensitive/wallets"])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.stdin(Stdio::null())
			.spawn();
			});
	} 

		loop{
			//redeclare the client object within the new scope
			let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
			let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
				Ok(client)=> client,
				Err(err)=> return format!("{}", err.to_string())
			};
			//query getblockchaininfo
			match client.get_blockchain_info(){
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
				Err(_) => {
					//sleep and continue the loop
					std::thread::sleep(Duration::from_secs(5));
					continue;
				},
			};
		}
		format!("SUCCESS in starting bitcoin daemon")
	}


#[tauri::command]
pub async fn stop_bitcoind() -> String {
	//start bitcoin daemon with networking inactive
	let output = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli")).args(["stop"]).output().unwrap();
	if !output.status.success() {
		
		return format!("ERROR in stopping bitcoin daemon = {}", std::str::from_utf8(&output.stderr).unwrap());
	}
	//sleep for 5 seconds before resolving
	std::thread::sleep(Duration::from_secs(5));

	format!("SUCCESS in stopping the bitcoin daemon")
}

// ./bitcoin-cli getdescriptorinfo '<descriptor>'
// analyze a descriptor and report a canonicalized version with checksum added
//acceptable params here are "low", "immediate", "delayed"
//this may not be useful for anything besides debugging on the fly
#[tauri::command]
pub async fn get_descriptor_info(walletname: String) -> String {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
		Ok(client)=> client,
		Err(err)=> return format!("{}", err.to_string())
	};
	//read descriptor to a string from file
	let desc: String = match fs::read_to_string(&("/mnt/ramdisk/sensitive/descriptors/".to_string()+&(walletname.to_string())+"_descriptor")){
		Ok(desc)=> desc,
		Err(err)=> return format!("{}", err.to_string())
	};
	let desc_info = client.get_descriptor_info(&desc).unwrap();
	format!("SUCCESS in getting descriptor info {:?}", desc_info)
}

#[tauri::command]
pub async fn load_wallet(walletname: String, hwnumber: String) -> Result<String, String> {
	//sleep time to ensure daemon is running before making an RPC call
	std::thread::sleep(Duration::from_secs(5));
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("error connecting to client: {}", err.to_string()))
	};
	// load the specified wallet...using a match statement here throws a JSON RPC error that breaks the loop logic
	client.load_wallet(&(walletname.to_string()+"_wallet"+&(hwnumber.to_string())));
	// parse list_wallets in a continuous loop to verify when rescan is completed
	loop{
		let list = match client.list_wallets(){
			Ok(list)=> list,
			Err(err)=> return Ok(format!("error listing wallets: {}", err.to_string()))
		};
		let search_string = &(walletname.to_string()+"_wallet"+&(hwnumber.to_string()));
		//listwallets returns the wallet name as expected...wallet is properly loaded and scanned
		if list.contains(&search_string){
			break;
		}
		//listwallets does not return the wallet name...wallet not yet loaded
		else{
			std::thread::sleep(Duration::from_secs(5));
		}
	}
	Ok(format!("Success in loading {} wallet", walletname))
	}

#[tauri::command]
pub async fn get_blockchain_info() -> String {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
		Ok(client)=> client,
		Err(err)=> return format!("{}", err.to_string())
	};
	//get blockchain info
	let info = client.get_blockchain_info();
	format!("Results: {:?}", info)
}

#[tauri::command]
pub async fn export_psbt(progress: String) -> String{
	// sleep for 4 seconds
	Command::new("sleep").args(["4"]).output().unwrap();
	//create conf for transfer CD
	let a = std::path::Path::new("/mnt/ramdisk/psbt/config.txt").exists();
	if a == false{
		let file = File::create(&("/mnt/ramdisk/psbt/config.txt")).unwrap();
		let output = Command::new("echo").args(["-e", &("psbt=".to_string()+&progress.to_string())]).stdout(file).output().unwrap();
		if !output.status.success() {
			return format!("ERROR with creating config: {}", std::str::from_utf8(&output.stderr).unwrap());
		}
	}
	let b = std::path::Path::new("/mnt/ramdisk/psbt/masterkey").exists();
	//copy over masterkey
	if b == false{
		let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/masterkey", "/mnt/ramdisk/psbt"]).output().unwrap();
		if !output.status.success() {
			return format!("ERROR with copying masterkey = {}", std::str::from_utf8(&output.stderr).unwrap());
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

//this is diffent from sign_funded_psbt in that this function is used to sign for a psbt that has already been signed with another wallet and expects the 
//WalletProcessPsbtResult struct rather than the WalletCreateFundedPsbtResult struct. PSBT originates from transfer CDROM here. 
#[tauri::command]
pub async fn sign_processed_psbt(walletname: String, hwnumber: String, progress: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//import the psbt from CDROM
	let psbt_str: String = match fs::read_to_string("/mnt/ramdisk/CDROM/psbt"){
		Ok(psbt_str)=> psbt_str,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//attempt to sign the tx
	let signed_result = client.wallet_process_psbt(
		&psbt.psbt,
		Some(true),
		None,
		None,
	);
	let signed = match signed_result{
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("Could not sign processed PSBT: {}", err.to_string()))
	};
	let a = std::path::Path::new("/mnt/ramdisk/psbt").exists();
	if a == false {
		let output = Command::new("mkdir").args(["/mnt/ramdisk/psbt"]).output().unwrap();
		if !output.status.success() {
		return Ok(format!("ERROR in creating /mnt/ramdisk/psbt dir {}", std::str::from_utf8(&output.stderr).unwrap()));
		}
	}
	//declare file dest
	let file_dest = "/mnt/ramdisk/psbt/psbt".to_string();
	//remove stale psbt from /mnt/ramdisk/psbt/psbt
	Command::new("sudo").args(["rm", "/mnt/ramdisk/psbt/psbt"]).output().unwrap();
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
		return Ok(format!("ERROR in sign_processed_psbt with creating config = {}", std::str::from_utf8(&output.stderr).unwrap()));
	}

	Ok(format!("Success in signing: {:?}", signed))
}

#[tauri::command]
//this is different than sign_processed_psbt in that it is used to sign for the first key in the quorum which will be in the WalletCreateFundedPsbtResult format rather
//than the WalletProcessPsbtResult format used in other circumstances. PSBT originates from RAM here.
//TODO maybe refactor sign_processed_psbt to look for either situation and act accordingly
pub async fn sign_funded_psbt(walletname: String, hwnumber: String, progress: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("Error establishing client connection: {}", err.to_string()))
	};
	//read the psbt from file
	let psbt_str: String = match fs::read_to_string("/mnt/ramdisk/psbt/psbt"){
		Ok(psbt_str)=> psbt_str,
		Err(err)=> return Ok(format!("Error reading PSBT from file: {}", err.to_string()))
	};
	//convert result to WalletCreateFundedPsbtResult
	let psbt: WalletCreateFundedPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("Error parsing PSBT: {}", err.to_string()))
	};

	//attempt to sign the tx
	let signed_result = client.wallet_process_psbt(
		&psbt.psbt,
		Some(true),
		None,
		None,
	);
	let signed = match signed_result{
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("Error signing PSBT: {}", err.to_string()))
	};
	//remove the stale psbt
	Command::new("sudo").args(["rm", "/mnt/ramdisk/psbt/psbt"]).output().unwrap();
	//declare file dest
	let file_dest = "/mnt/ramdisk/psbt/psbt".to_string();
	//remove stale psbt from /mnt/ramdisk/psbt/psbt
	Command::new("sudo").args(["rm", "/mnt/ramdisk/psbt/psbt"]).output().unwrap();
	//store the signed transaction as a file
	match store_psbt(&signed, file_dest) {
	Ok(_) => {},
	Err(err) => return Err("ERROR could not store PSBT: ".to_string()+&err)
	};

	Ok(format!("Reading PSBT from file: {:?}", signed))
}

//This function is redundant and can be removed
#[tauri::command]
pub async fn finalize_psbt(walletname: String, hwnumber: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//copy the psbt from CDROM to /mnt/ramdisk/psbt/ if necessary
	let a = std::path::Path::new("/mnt/ramdisk/CDROM/psbt").exists();
	let b = std::path::Path::new("/mnt/ramdisk/psbt/psbt").exists();
	if a == true && b == false{
		Command::new("mkdir").arg("/mnt/ramdisk/psbt").output().unwrap();
		let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/psbt", "/mnt/ramdisk/psbt"]).output().unwrap();
			if !output.status.success() {
			return Ok(format!("ERROR in psbt from CDROM dir to psbt dir{}", std::str::from_utf8(&output.stderr).unwrap()));
			}
	}
	//read psbt to string from a file
	let psbt_str: String = match fs::read_to_string("/mnt/ramdisk/psbt/psbt"){
		Ok(psbt_str)=> psbt_str,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};




	//TODO can remove this once the wrapped cli is implemented
	//finalize the tx
	// let finalized_result = client.finalize_psbt(
	// 	&psbt.psbt,
	// 	None,
	// );



	//TODO wrap cli command for finalizepsbt here instead of the above
	let finalized_result = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli"))
	.args([ "finalizepsbt", 
	&psbt.psbt.to_string()
	]) 
	.output()
	.unwrap();
	//TODO wrap cli command for sendrawtransaction in the broadcast function

	if !finalized_result.status.success() {
		return Ok(format!("ERROR in finalizing PSBT = {}", std::str::from_utf8(&finalized_result.stderr).unwrap()));
	}
	let finalized = String::from_utf8(finalized_result.stdout).unwrap();
	// let finalized = match finalized_str{
	// 	Ok(psbt)=> psbt,
	// 	Err(err)=> return Ok(format!("{}", err.to_string()))
	// };
	// if finalized_result.stdout.complete == false{
	// 	return Ok(format!("ERROR PSBT not complete"))
	// }


	
	Ok(format!("Reading PSBT from file: {:?}", finalized))
}

#[tauri::command]
pub async fn broadcast_tx(walletname: String, hwnumber: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//copy the psbt from CDROM to /mnt/ramdisk/psbt/ if necessary
	let a = std::path::Path::new("/mnt/ramdisk/CDROM/psbt").exists();
	let b = std::path::Path::new("/mnt/ramdisk/psbt/psbt").exists();
	if a == true && b == false{
		Command::new("mkdir").arg("/mnt/ramdisk/psbt").output().unwrap();
		let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/psbt", "/mnt/ramdisk/psbt"]).output().unwrap();
			if !output.status.success() {
			return Ok(format!("ERROR in psbt from CDROM dir to psbt dir{}", std::str::from_utf8(&output.stderr).unwrap()));
			}
	}
	//read the psbt from file
	let psbt_str: String = match fs::read_to_string("/mnt/ramdisk/psbt/psbt"){
		Ok(psbt_str)=> psbt_str,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//finalize the psbt
	let finalized_result = client.finalize_psbt(
		&psbt.psbt,
		None,
	);
	let finalized = match finalized_result{
		Ok(tx)=> tx.hex.unwrap(),
		Err(err)=> return Ok(format!("{}", err.to_string()))	
	};
	let finalized_str = hex::encode(finalized);

	//broadcast the tx
	let broadcast_output = Command::new(&(get_home()+"/bitcoin-25.0/bin/bitcoin-cli"))
		.args([&("-rpcwallet=".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), 
		"sendrawtransaction", 
		&finalized_str
		]) 
		.output()
		.unwrap();
		if !broadcast_output.status.success() {
			return Ok(format!("ERROR in broadcasting PSBT = {}", std::str::from_utf8(&broadcast_output.stderr).unwrap()));
		}
		//convert psbt to string from hex
		let broadcast = String::from_utf8(broadcast_output.stdout).unwrap();





	//broadcast the tx
	// let broadcast_result = client.send_raw_transaction(&finalized[..]);
	// let broadcast = match broadcast_result{
	// 	Ok(tx)=> tx,
	// 	Err(err)=> return Ok(format!("{}", err.to_string()))
	// };



	//remove stale psbt from ramdisk
	Command::new("sudo").args(["rm", "-r", "/mnt/ramdisk/psbt"]).output().unwrap();
	Ok(format!("Broadcasting Fully Signed TX: {:?}", broadcast))
}

//used to decode a PSBT and display tx parameters on the front end
#[tauri::command]
pub async fn decode_processed_psbt(walletname: String, hwnumber: String) -> Result<String, String>{
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//copy the psbt from CDROM to /mnt/ramdisk/psbt/ if necessary
	let a = std::path::Path::new("/mnt/ramdisk/CDROM/psbt").exists();
	let b = std::path::Path::new("/mnt/ramdisk/psbt/psbt").exists();
	if a == true && b == false{
		Command::new("mkdir").arg("/mnt/ramdisk/psbt").output().unwrap();
		let output = Command::new("cp").args(["/mnt/ramdisk/CDROM/psbt", "/mnt/ramdisk/psbt"]).output().unwrap();
			if !output.status.success() {
			return Ok(format!("ERROR in psbt from CDROM dir to psbt dir{}", std::str::from_utf8(&output.stderr).unwrap()));
			}
	}
	//read the psbt from file
	let psbt_str: String = match fs::read_to_string("/mnt/ramdisk/psbt/psbt"){
		Ok(psbt_str)=> psbt_str,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//convert result to valid base64
	let psbt: WalletProcessPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//decode the psbt
	let psbt_bytes = base64::decode(&psbt.psbt).unwrap();
	let psbtx: PartiallySignedTransaction = PartiallySignedTransaction::deserialize(&psbt_bytes[..]).unwrap();
	// Calculate the total fees for the transaction
	let fee_amount = psbtx.fee().unwrap();
	let fee = fee_amount.to_btc();


	//establish a baseline index for the output vector
	let mut x = 0;
	let length = psbtx.unsigned_tx.output.len();

	//attempt to filter out change output
	while length > x {
		//obtain scriptpubkey for output at index x
		let script_pubkey = psbtx.unsigned_tx.output[x].script_pubkey.as_script(); 

		//obtain amount of output
		let amount = psbtx.unsigned_tx.output[x].value;

		//derive address from scriptpubkey
		let address = match bitcoin::Address::from_script(script_pubkey, bitcoin::Network::Bitcoin){
			Ok(address)=> address,
			Err(err)=> return Ok(format!("{}", err.to_string()))
        };

		//check if address ismine: true
		let address_info_result: Result<bitcoincore_rpc::json::GetAddressInfoResult, bitcoincore_rpc::Error> = client.call("getaddressinfo", &[address.to_string().into()]); 

        let address_info = match address_info_result {
			Ok(info)=>info,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};

		//if the address is not recognized, return the results
		if address_info.is_mine == Some(false) {
			return Ok(format!("address={:?}, amount={:?}, fee={:?}", address, amount, fee))
		//else continue to iterate through the vector
		}else{
			x += 1;
		}
	}

	//fallback if the user is sending to their own wallet
	//obtain scriptpubkey for output at index 0
	let script_pubkey = psbtx.unsigned_tx.output[0].script_pubkey.as_script(); 

	//obtain amount of output
	let amount = psbtx.unsigned_tx.output[0].value;

	//derive address from scriptpubkey
	let address = match bitcoin::Address::from_script(script_pubkey, bitcoin::Network::Bitcoin){
		Ok(address)=> address,
		Err(err)=> return Ok(format!("{}", err.to_string()))
    };

	Ok(format!("address={:?}, amount={:?}, fee={:?}", address, amount, fee))
}

//used to decode a walletcreatefundedpsbt result
#[tauri::command]
pub async fn decode_funded_psbt(walletname: String, hwnumber: String) -> Result<String, String> {
	let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&("127.0.0.1:8332/wallet/".to_string()+&(walletname.to_string())+"_wallet"+&hwnumber.to_string()), auth){
		Ok(client)=> client,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};
	//check if this file path exists
	let a = std::path::Path::new("/mnt/ramdisk/psbt/psbt").exists();
	let psbt_str: String;
	//if it does exist, read the psbt from file
	if a == true{
		psbt_str = match fs::read_to_string("/mnt/ramdisk/psbt/psbt"){
			Ok(psbt_str)=> psbt_str,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
		//if it doesn't exist then we can assume the psbt is still on the transfer CD
	}else{
		psbt_str = match fs::read_to_string("/mnt/ramdisk/CDROM/psbt"){
			Ok(psbt_str)=> psbt_str,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};
	}


	//convert result to WalletCreateFundedPsbtResult
	let psbt: WalletCreateFundedPsbtResult = match serde_json::from_str(&psbt_str) {
		Ok(psbt)=> psbt,
		Err(err)=> return Ok(format!("{}", err.to_string()))
	};

	//calculate the fee 
	let fee = psbt.fee.to_btc();

	//convert the byte slice to a PartiallySignedTransaction Struct
	let psbt_bytes = base64::decode(&psbt.psbt).unwrap();
	let psbtx: PartiallySignedTransaction = PartiallySignedTransaction::deserialize(&psbt_bytes[..]).unwrap();

	//establish a baseline index for the output vector
	let mut x = 0;
	let length = psbtx.unsigned_tx.output.len();

	//attempt to filter out change output
	while length > x {

		//obtain amount of output
		let amount = psbtx.unsigned_tx.output[x].value;

		//obtain scriptpubkey for output at index x
		let script_pubkey = psbtx.unsigned_tx.output[x].script_pubkey.as_script(); 

		//derive address from scriptpubkey
		let address = match bitcoin::Address::from_script(script_pubkey, bitcoin::Network::Bitcoin){
			Ok(address)=> address,
			Err(err)=> return Ok(format!("{}", err.to_string()))
        };

		//check if address ismine: true
		let address_info_result: Result<bitcoincore_rpc::json::GetAddressInfoResult, bitcoincore_rpc::Error> = client.call("getaddressinfo", &[address.to_string().into()]); 

        let address_info = match address_info_result {
			Ok(info)=>info,
			Err(err)=> return Ok(format!("{}", err.to_string()))
		};

		//if the address is not recognized, return the results
		if address_info.is_mine == Some(false) {
			return Ok(format!("address={:?}, amount={:?}, fee={:?}", address, amount, fee))
		//else continue to iterate through the vector
		}else{
			x += 1;
		}
	}

	//fallback if the user is sending to their own wallet
	//obtain scriptpubkey for output at index 0
	let script_pubkey = psbtx.unsigned_tx.output[0].script_pubkey.as_script(); 

	//obtain amount of output
	let amount = psbtx.unsigned_tx.output[0].value;

	//derive address from scriptpubkey
	let address = match bitcoin::Address::from_script(script_pubkey, bitcoin::Network::Bitcoin){
		Ok(address)=> address,
		Err(err)=> return Ok(format!("{}", err.to_string()))
    };

	Ok(format!("address={:?}, amount={:?}, fee={:?}", address, amount, fee))
}

//retrieve current median block time
#[tauri::command]
pub fn retrieve_median_blocktime() -> String{
    let auth = bitcoincore_rpc::Auth::UserPass("rpcuser".to_string(), "477028".to_string());
    let client = match bitcoincore_rpc::Client::new(&"127.0.0.1:8332".to_string(), auth){
		Ok(client)=> client,
		Err(err)=> return format!("{}", err.to_string())
	};
    let time_med = client.get_blockchain_info().unwrap().median_time;
	// let time_parsed: u64 = time_med.parse();
	let time = time_med - 1000;
    format!("{}", time.to_string())
}



//simulate time machine, take the user all the way to broadcast
#[tauri::command]
pub fn simulate_time_machine() -> Result<String, String>{
	//in the future, the user will have to obtain the time machine xprivs (or descriptors)
	//from the time machine operator and bring those xprivs back to their machine in order to construct the proper time machine descriptors & wallets

	//ALL these steps need to happen before this function fires...
	//1. obtain the psbt from the transfer CD
	// media/ubuntu/CDROM/psbt
	//2. prompt the user to insert the setup CD (needed to obtain the time machine keys).
	//3. copy setup CD to ramdisk

	//TODO potentially, rather than building the descriptors here...
	//let's build them at create_descriptor stage of things and keep them with the timemachinekeys, they can be encrypted while stored with the BPS


	//obtain the delayed descriptor from sensitive
	// /mnt/ramdisk/sensitive/descriptors/delayed_descriptor1

	//obtain the HW1 xpub
	// /mnt/ramdisk/sensitive/public_key1

	//modify the descriptor to use HW1 xpub

	//obtain the time_machine_xpriv1
	// media/ubuntu/CDROM/timemachinekeys/time_machine_private_key1

	//obtain the time_machine_xpriv2
	// media/ubuntu/CDROM/timemachinekeys/time_machine_private_key2

	//modify the descriptor to use Time_machine_xpriv1 and output as time_machine_descriptor1

	//modify the descriptor to use time_machine_xpriv2 and output as time_machine_descriptor2

	//create blank time_machine1 wallet

	//create blank time_machine2 wallet

	//import time_machine_descriptor1 into time_machine1 wallet

	//sign the psbt with time_machine1 wallet

	//import time_machine_descriptor2 into time_machine2 wallet

	//sign the new psbt with time_machine2 wallet

	//finalize the psbt and output to where delayedBroadcast expects to find it

	//take the user to broadcast (front end can handle this)

	Ok(format!("success"))
}
