<template>
    <div class="container" style="margin-top: 3rem;">
      <h2>Arctica. A secure & private Bitcoin cold storage solution</h2>
      <p><b>WARNING: WE ARE CURRENTLY IN ALPHA TESTING, DO NOT USE ARCTICA FOR THE STORAGE OF FUNDS
       <br>this software overwrites external storage media and CDs without much warning, I advise you only run arctica on a dedicated machine, remove any extraneous external storage media, and only insert new/blank USB sticks or SD cards and CDs when prompted.</b> </p>
      <p><b>Minimum Specs</b>: Extensive testing has shown that arctica does not perform well on very low end computers. This is due to the demanding nature of running the Operating System from the usb sticks or sd cards. The absolute bare minimum I have been able to get arctica to work properly on are 6GB of RAM and a dual core AMD Ryzen 3 3200U. However, this made for an extremely frustrating user experience (very slow & buggy) and I highly reccomend overshooting these minimum specs if at all possible. For example, a laptop with 16GB of RAM and a quad core i7-6700HQ @ 2.6GHz runs arctica extremely well. Additionally you need atleast a 1TB internal storage drive for the bitcoin blockchain. I reccomend replacing the laptops internal storage drive with an aftermarket SSD drive to improve the initial sync speed significantly. 
      <p>The computer's internal storage should be flashed with a clean installation of the latest Ubuntu release prior to installing Arctica.</p>
      <p>NOTE: Arctica is currently in Alpha and is not currently reccomended for the storage of funds. This is a spiritual successor to <a href="https://github.com/JWWeatherman/yeticold">Yeti Cold</a>, which is my reccomended Bitcoin storage protocol until Arctica releases a Beta client</p>
      <p><a href="https://github.com/wild-kard/arctica-frontend">Front End Repo</a></p>
      <p>A comprehensive technial design document can be found <a href="https://docs.google.com/document/d/1_RZysHjRNKTzPG_xDWh8-EvLn57AOlBO3d9J-_0bSRQ/edit?usp=sharing">here</a> </p>
        <p> <a href="https://www.figma.com/file/KcE9byRVhSntYcTITn1OvY/Bitcoin-Wallet-UI-Kit-(Arctica)?node-id=3350%3A85090">User Experience Design Documents </a></p>
      <p>Arctica is a Free and Open Source wrapper script that installs bitcoin core and then walks the user through setup of a highly secure & private cold storage solution. The software is designed to make Bitcoin more difficult to lose, steal, or extort than any other asset. This protocol contains both a high security and a medium security area and is designed for storage of amounts in excess of $100,000.</p>
      <ul>
          <li>Arctica is built in Rust on top of <a href="https://github.com/bicoindevkit">BDK</a>. The front end is built with Vue.js and runs as a standalone desktop application through tauri which emulates web view without requiring the use of a browser.</li>
          <li>Arctica requires users do what is needed for safe and secure bitcoin storage even when this requires more time and effort - the first task in the Arctica instructions is to setup trustworthy & dedicated Bitcoin laptops.</li>
          <li>Private keys are never on any device with a channel to an Internet connected device except through SD cards which are loaded into RAM through Linux Live System. Although the use of QR codes would be optimal, bitcoin core does not support offline signing via QR codes and the additional attack surface introduced to support this might outweigh the benefits. The purpose of an "air gap" is to limit the amount of data that can be moved, limit the times data can be moved, and make it easy to verify the data is accurate "out of band" before sending. SD cards are inferior to QR codes in all of these areas, but the risk that a QR code library has a security flaw must be weighed against these advantages. Additionally, we use SD cards to create a seamless OS environment. </li>
          <li>Artica uses both an ecrypted 5 of 7 & 2 of 7 decaying multisig for bitcoin storage. This allows up to 6 keys to be lost without losing bitcoin and requires 5 locations to be compromised by an attacker to lose privacy or funds. This prioritizes recovery redundancy and privacy.</li>
          <li>HD Multisig is used so that you can send funds to 1,000 addresses, but recover all funds using only 5 seed phrases (high security) or 2 seed phrases (medium security), both of which eventually decay down to 1 of 7 (low security) after a predetermined time frame.</li>
          <li>Generic computing hardware is used. Hardware sold specifically for bitcoin storage requires trusting all parties from manufacturing to shipping. Omitting potential for modified Btcoin specific hardware to steal bitcoin. However, support for Hardware wallets for use as a signing device for a number of keys in the 7 key quorom is planned.</li>
          <li>Minimal software beyond bitcoin core. Bitcoin core is far and away the most trustworthy bitcoin software. Unfortunately it does not yet provide a user friendly interface for establishing a multisig address or display and accept private keys in a human writable format. We have intentionally sought to limit dependencies on external software libraries in our design process. Ideally, an Arctica user could recover their funds without our software and only use bitcoin core (if they had a working knowledge of the Bitcoin CLI and BDK/miniscript)</li>
          <li>Open source and easily audited. One of the reasons bitcoin core is trustworthy is that it is the most scrutinized software. This makes it the least likely to contain a critical security flaw that has not been identified and fixed. Arctica will never be as trustworthy, but by minimizing the amount of code and primarily using Rust, the BDK, and console commands the effort required to verify that Arctica is performing as expected is minimized.</li>
          <li>Usable for non-technical users. By following simple instructions users with moderate computer literacy can use Arctica. This is important because trusting someone to help you establish your cold storage solution introduces considerable risk. We want Arctica to be the gold standard for newcomers to bitcoin to establish a secure self custody profile.</li>
          <li>Private keys & descriptors are stored in non-descript and encrypted packages.</li>
          <li>Private. Unlike many popular hardware and software wallets that transmit your IP address (home address) and bitcoin balance to third party servers, Arctica uses a bitcoin core full node. This means nothing is shared beyond what is required to create a bitcoin transaction. Arctica will also use Tor (planned for v2).</li>
          <li>Counterfeit prevention. The only way to be certain that your balance represents genuine bitcoin is to use a bitcoin full node - in fact that is the primary purpose of a bitcoin full node - to verify that the bitcoin balance is correct and full of only genuine bitcoins. Any solution that does not involve a full node requires you trust someone else to tell you if you have real bitcoin.</li>
          <Li>Minimal hardware. You only need access to two relatively cheap computers. Or one moderate computer and one cheap computer. These computers will be dedicated to the purpose of running arctica. If you don't own a laptop you can buy one from a big box store and return it after setup is completed. Once Arctica is set up, it will work on any computer by a user simply inserting an Arctica SD card and rebooting, however the bitcoin blockchain might need to be redownloaded on a new or different computer.</li>
          <li>The prompts are designed to be completed by non technologists with minimal effort.</li>
          <li>Bitcoin private keys (Arctica's SD cards or USB sticks along with CD backups) are stored on encrypted SD cards in multiple geographic locations.</li>
          <li>Software instructions for recovering and spending the bitcoin are included with on every SD card to reduce the likelihood of loss and improve UX.</li>
      </ul>
      <p>Arctica provides the best balance of security, ease of use and privacy when storing significant sums of bitcoin, it has the following disadvantages that might not be expected:</p>
      <ul>
          <li>Time. To complete setup you will need to invest several hours spread over the course of a couple days. This time includes active participation in setting up devices by following on screen prompts, syncing the blockchain, and establishing a series of security protocols.</li>
          <li>Soft Shelf Life. Because Arctica is designed to have a decaying high & medium security storage area, you will find that Arctica's security assurances intentionally degrade over time. This decision has been taken to find a balance between high security assurance and inheritance in the event of a users untimely demise. A user is advised to repeat Arctica setup shortly before or during the 4 year threshold decay.</li>
          <li>Privacy. While using bitcoin core over Tor does provide significant privacy advantages over many cold storage solutions, using multisig is not very common. This means that someone could look at the blockchain and infer that the owner of the coins is probably using our software for cold storage. This will eventually be fixed through changes to bitcoin and it is worth the security and recovery benefit to use multisig and the type of multisig you are using is only exposed to the network when you spend from Arctica (not when you deposit funds).</li>
        </ul>
    </div>
</template>

<u>Dev notes:</u>

<b>First time installation</b>

To build arctica from source first install the latest rustup toolchain

clone the git repo in your home directory

`git clone https://github.com/wild-kard/arctica`

Navigate into the arctica directory from your home directory

`cd arctica`

Run the first time submodule install 

`git submodule update --init --recursive`

Install tauri dependencies

`sudo apt update`
`sudo apt install libwebkit2gtk-4.0-dev`
`sudo apt install build-essential`
`sudo apt install curl`
`sudo apt install wget`
`sudo apt install libssl-dev`
`sudo apt install libgtk-3-dev`
`sudo apt install libayatana-appindicator3-dev`
`sudo apt install librsvg2-dev`

Compile front end first

`cd arctica-frontend`
`npm install`
`npm run build`

compile backend (you must do this if building from source)

`cd ..`
`cargo build`

run the application and start following the prompts

`sudo cargo run`

<b>Installing updates</b>

submodule updates

`git submodule update --recursive --remote`

navigate to the front end

`cd arctica-frontend`

compile front end

`npm run build`

return to the main directory

`cd ..`

pull down the latest for the backend

`git pull`

compile binary 

`cargo build`


run the app

`sudo cargo run`