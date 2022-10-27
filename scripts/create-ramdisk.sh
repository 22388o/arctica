sudo mkdir /mnt/ramdisk

sudo mount -t ramfs -o size=10M ramfs /mnt/ramdisk

#make target dir for encrypted payload to or from SD cards
mkdir /mnt/ramdisk/sensitive