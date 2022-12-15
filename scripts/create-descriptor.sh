#create a simulated descriptor
echo "this is a simulated descriptor" > /mnt/ramdisk/sensitive/descriptor.txt

#export descriptor to setupCD
cp /mnt/ramdisk/sensitive/descriptor.txt /mnt/ramdisk/CDROM/