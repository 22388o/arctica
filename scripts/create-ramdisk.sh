sudo mkdir /mnt/ramdisk

sudo mount -t ramfs -o size=10M ramfs /mnt/ramdisk

#make target dir for encrypted payload to or from SD cards
sudo mkdir /mnt/ramdisk/sensitive
sudo chmod 777 /mnt/ramdisk/sensitive