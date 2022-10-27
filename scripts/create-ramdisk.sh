sudo mkdir /mnt/ramdisk
sudo chmod 777 /mnt/ramdisk

sudo mount -t ramfs -o size=1G ramfs /mnt/ramdisk

#make target dir for encrypted payload to or from SD cards
sudo mkdir /mnt/ramdisk/sensitive
sudo chmod 777 /mnt/ramdisk/sensitive