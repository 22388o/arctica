FILE="./ubuntu.iso"
rm persistent-xubuntu2.iso
rm persistent-xubuntu3.iso
if [ ! -f "$FILE" ]; then 
    wget -O ubuntu.iso http://releases.ubuntu.com/jammy/ubuntu-22.04.1-desktop-amd64.iso
fi
< ubuntu.iso sed 's/maybe-ubiquity/  persistent  /' > persistent-xubuntu2.iso
< persistent-xubuntu2.iso sed 's/set timeout=30/set timeout=1 /' > persistent-xubuntu3.iso
fallocate -l 5GiB persistent-xubuntu3.iso
kvm -drive file=persistent-xubuntu3.iso -m 8192 -daemonize -pidfile pid.txt
sleep 100
kill -9 $(cat ./pid.txt)
udisksctl loop-setup -f persistent-xubuntu3.iso
sleep 2
sudo mkdir /media/a/writable/upper/home/ubuntu/test
