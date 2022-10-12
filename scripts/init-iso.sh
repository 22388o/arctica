sudo apt-get -y install qemu-system-x86 qemu-kvm libvirt-clients libvirt-daemon-system bridge-utils

FILE="./ubuntu-22.04.1-desktop-amd64.iso"
FILE1="./bitcoin-23.0-x86_64-linux-gnu.tar.gz"
#obtain blk id for internal storage for symlinking .bitcoin folders
UUID=$(echo $(blkid) | cut -d '"' -f 2)
if [ ! -f "$FILE" ]; then 
    wget -O ubuntu-22.04.1-desktop-amd64.iso http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso
fi

if [ ! -f "$FILE1" ]; then
    wget https://bitcoincore.org/bin/bitcoin-core-23.0/bitcoin-23.0-x86_64-linux-gnu.tar.gz
fi

sudo rm persistent-ubuntu.iso
sudo rm persistent-ubuntu1.iso
sudo rm pid.txt
< ubuntu-22.04.1-desktop-amd64.iso sed 's/maybe-ubiquity/  persistent  /' > persistent-ubuntu1.iso
< persistent-ubuntu1.iso sed 's/set timeout=30/set timeout=1 /' > persistent-ubuntu.iso
sudo rm persistent-ubuntu1.iso
fallocate -l 5GiB persistent-ubuntu.iso
kvm -m 2048 ~/arctica/persistent-ubuntu.iso -daemonize -pidfile pid.txt -cpu host -display none
sleep 200
kill -9 $(cat ./pid.txt)
udisksctl loop-setup -f persistent-ubuntu.iso
sleep 2
#copy over artica binary
sudo cp ~/arctica/target/debug/app /media/$USER/writable/upper/home/ubuntu/arctica
sudo cp ~/arctica/icons/arctica.jpeg /media/$USER/writable/upper/home/ubuntu/arctica.jpeg
sudo cp ~/arctica/shortcut/Arctica.desktop /media/$USER/writable/upper/usr/share/applications/Arctica.desktop
sudo chmod +x /media/$USER/writable/upper/usr/share/applications/Arctica.desktop
#extract bitcoin core
sudo tar -xzf bitcoin-23.0-x86_64-linux-gnu.tar.gz -C /media/$USER/writable/upper/home/ubuntu
#make local internal bitcoin dotfile
sudo mkdir --parents /home/$USER/.bitcoin/blocks /home/$USER/.bitcoin/chainstate
#create device .bitcoin dir
sudo mkdir /media/$USER/writable/upper/home/ubuntu/.bitcoin
#symlink chainstate
sudo ln -s ~/.bitcoin/chainstate /media/ubuntu/$UUID/.bitcoin/chainstate
#symlink blockdata
sudo ln -s ~/.bitcoin/blocks /media/ubuntu/$UUID/.bitcoin/blocks
#download mkusb deps
sudo add-apt-repository -y universe
sudo add-apt-repository -y ppa:mkusb/ppa
sudo apt -y update
sudo apt install -y mkusb
sudo apt install -y usb-pack-efi
