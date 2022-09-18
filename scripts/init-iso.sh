FILE="./ubuntu.iso"
if [ ! -f "$FILE" ]; then 
    wget -O ubuntu-22.04.1-desktop-amd64.iso http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso
fi
< ubuntu-22.04.1-desktop-amd64.iso sed 's/maybe-ubiquity/  persistent  /' > persistent-ubuntu.iso
< persistent-ubuntu.iso sed 's/set timeout=30/set timeout=1 /' > persistent-ubuntu.iso
fallocate -l 5GiB persistent-ubuntu.iso
kvm -drive file=persistent-ubuntu.iso -m 8192 -daemonize -pidfile pid.txt
sleep 100
kill -9 $(cat ./pid.txt)
udisksctl loop-setup -f persistent-ubuntu.iso
sleep 2
sudo mkdir /media/$USER/writable/upper/home/ubuntu/test
