# Arctica: A Secure and Private Bitcoin Cold Storage Solution

NOTE: A spiritual successor to <a href="https://github.com/JWWeatherman/yeticold">Yeti Cold</a>, this project is currently in alpha. **Do not use this code for storage of funds until a beta version is released.**

## What is Arctica?

Arctica is a Free and Open Source wrapper script that installs Bitcoin Bore and walks the user through setup of a highly secure and private cold storage solution. The software is designed to make bitcoin more difficult to lose, steal, or extort than any other asset. This protocol contains both high security and medium security areas and is designed for storage of amounts in excess of $100,000.

A comprehensive technial design document can be found <a href="https://docs.google.com/document/d/1_RZysHjRNKTzPG_xDWh8-EvLn57AOlBO3d9J-_0bSRQ/edit?usp=sharing">here</a>.

## Work in Progress

- <a href="https://www.figma.com/file/KcE9byRVhSntYcTITn1OvY/Bitcoin-Wallet-UI-Kit-(Arctica)?node-id=3350%3A85090">User Experience Design Documents </a> (Design and UX flow is currently the main focus of development. If you would like to help contribute please send me a message or open an issue and I can make you an editor on the design document. We are currently using Figma and the Free and Open Source Bitcoin UI Kit Assets.)

- <a href="https://github.com/wild-kard/arctica-frontend">Front End Repo</a>

## Advantages of Arctica

 - Arctica requires users do what is needed for safe and secure bitcoin storage even when this requires more time and effort - the first task in the Arctica instructions is to setup trustworthy & dedicated Bitcoin laptops.

 - Private keys are never on any device with a channel to an internet-connected device, except through SD cards which are loaded into RAM through Linux Live System. Although the use of QR codes would be optimal, Bitcoin Core does not support offline signing via QR codes, and the additional attack surface introduced to support QR codes might outweigh the benefits. The purpose of an "air gap" is to limit the amount of data that can be moved, limit the times data can be moved, and make it easy to verify the data is accurate "out of band" before sending. SD cards are inferior to QR codes in all of these areas, but the risk that a QR code library has a security flaw must be weighed against these advantages. Additionally, we use SD cards to create a seamless OS environment.
 - Artica uses an ecrypted 5-of-7 decaying multisig for bitcoin storage. This allows up to 6 keys to be lost without losing bitcoin and requires 5 locations to be compromised by an attacker to lose privacy or funds. This encrypted multisig scheme prioritizes recovery redundancy and privacy.
 - HD Multisig is used so that you can send funds to 1,000 addresses, but recover all funds using only 5 seed phrases (high security area) or 2 seed phrases (medium security area), which eventually decay down to 1 of 7 after a predetermined time frame.
 - Generic computing hardware is used. Hardware sold specifically for bitcoin storage requires trusting everyone from manufacturing to shipping to fail to realize the opportunity available to modify the hardware in order to steal bitcoin. However, support for Hardware wallets for use as a signing device for a number of keys in the 7 key quorom is planned.
 - Minimal software beyond Bitcoin Core. Bitcoin Core is far and away the most trustworthy bitcoin software. Unfortunately, it does not yet provide a user-friendly interface for establishing a multisig address or displaying/accepting private keys in a human writable format. We have intentionally sought to limit dependencies on external software libraries in our design process. Ideally, an Arctica user could recover their funds without our software and only use Bitcoin Core (if they had a working knowledge of the Bitcoin CLI.)
 - Open source and easily audited. One aspect that makes Bitcoin Core trustworthy is that it is the most scrutinized software. This makes it the least likely to contain a critical security flaw that has not been identified and fixed. Arctica will never be as trustworthy, but the effort required to verify that Arctica is performing as expected is minimized by minimizing the amount of code and primarily using Rust, the BDK, and console commands.
 - Usable for non-technical users with minimal effort. By following simple instructions, users with moderate computer literacy can use Arctica. This is important because trusting someone to help you establish your cold storage solution introduces considerable risk. We want Arctica to be the gold standard for newcomers to bitcoin to establish a secure self custody profile.
 - Private keys & descriptors are stored in non-descript and encrypted packages.
 - Private. Unlike many popular hardware and software wallets that transmit your IP address (home address) and bitcoin balance to third party servers, Arctica uses a bitcoin full node. This means nothing is shared beyond what is required to create a bitcoin transaction. Arctica will also use Tor.
 - Counterfeit prevention. The only way to be certain that your balance represents genuine bitcoin is to use a bitcoin full node - in fact that is the primary purpose of a bitcoin full node - to verify that the bitcoin balance is correct and full of only genuine bitcoins. Any solution that does not involve a full node requires you trust someone else to tell you if you have real bitcoin.
 - Minimal hardware. You only need access to two cheap computers. If you don't own a laptop you can buy one from a big box store and return it after setup is completed.
 - Bitcoin private keys are stored on encrypted SD cards in multiple geographic locations.
 - Software instructions for recovering and spending the bitcoin are included with on every SD card to reduce the likelihood of loss and improve UX.

## Disadvantages of Arctica

While Arctica provides the best balance of security, privacy, and, ease of use when storing significant sums of bitcoin, it has the following disadvantages that might not be expected:
- Time. To complete setup you will need to invest several hours spread over the course of a couple days. This time includes writing down syncing the blockchain, flashing SD cards, and establishing a series of security protocols.

- Soft Shelf Life. Because Arctica is designed to have a decaying high security storage area, you will find that Arctica's security assurances intentionally degrade over time. This decision has been taken to find a balance between high security assurance and inheritance in the event of a users untimely demise.
- Privacy. While using Bitcoin Core over Tor does provide significant privacy advantages over many cold storage solutions, using multisig is not very common. This means that someone could look at the blockchain and infer that the owner of the coins is probably using our software for cold storage. This will eventually be fixed through changes to bitcoin. Multisig is still worth using in light of the privacy risk due to the security and recovery benefits. Additionally, the type of multisig you are using is only exposed to the network when you spend from Arctica (not when you deposit funds).

## Build from Source Instructions

1. Install Rust and dependencies.
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # taken from https://www.rust-lang.org/tools/install
    sudo apt update && sudo apt install -y nodejs npm libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
    ```
2. Clone the repository.
    
    Note: Update any submodule changes since cloning with `git submodule update --remote --recursive`
    ```bash
    git clone --recurse-submodules https://github.com/wild-kard/arctica.git
    ```
    

3. Compile the frontend first.
    ```bash
    cd arctica/arctica-frontend/
    npm install
    npm run build
    ```
4. Compile the backend.

    Note: If you see a Rust build error ``failed to parse the `edition` key``, then remove `rustc` and `cargo` then [re-install Rust](https://www.rust-lang.org/tools/install).
    ```bash
    cd ../
    cargo build
    ```
5. Run the application and begin following the prompts.
    ```bash
    cargo run
    ```

Here are the instructions repeated in one code block.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # taken from https://www.rust-lang.org/tools/install
sudo apt update && sudo apt install -y nodejs npm libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
git clone --recurse-submodules https://github.com/wild-kard/arctica.git
cd arctica/arctica-frontend/
npm install
npm run build
cd ../
cargo build
cargo run
``` 

WARNING: This software overwrites external storage media and CDs without much warning, I advise you only run arctica on a dedicated machine, remove any extraneous external storage media, and only use USB sticks or SD cards and CDs you don't mind having wiped.
