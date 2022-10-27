sudo mkdir /mnt/ramdisk
sudo mount -t ramfs -o size=1G ramfs /mnt/ramdisk
sudo chmod 777 /mnt/ramdisk

#make target dir for encrypted payload to or from SD cards
mkdir /mnt/ramdisk/sensitive
