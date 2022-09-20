sudo apt-get -y install qemu-system-x86 qemu-kvm libvirt-clients libvirt-daemon-system bridge-utils

FILE="./ubuntu-22.04.1-desktop-amd64.iso"
if [ ! -f "$FILE" ]; then 
    wget -O ubuntu-22.04.1-desktop-amd64.iso http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso
fi
< ubuntu-22.04.1-desktop-amd64.iso sed 's/maybe-ubiquity/  persistent  /' > persistent-ubuntu1.iso
< persistent-ubuntu1.iso sed 's/set timeout=30/set timeout=1 /' > persistent-ubuntu.iso
fallocate -l 5GiB persistent-ubuntu.iso
kvm -m 2048 ~/arctica/persistent-ubuntu.iso -daemonize -pidfile pid.txt
sleep 200
kill -9 $(cat ./pid.txt)
udisksctl loop-setup -f persistent-ubuntu.iso
sleep 2
sudo mkdir /media/$USER/writable/upper/home/ubuntu/test

sudo add-apt-repository -y universe
sudo add-apt-repository -y ppa:mkusb/ppa
sudo apt -y update
sudo apt install -y mkusb
sudo apt install -y usb-pack-efi
