#convert a completed recovery cd to a transfer cd

#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#make the transfer CD dir which holds files to be burned to the transfer CD
mkdir /mnt/ramdisk/transferCD

#create transferCD config
echo "type=transfercd" > /mnt/ramdisk/transferCD/config.txt

#collect masterkey from cd dump and prepare for transfer to transfercd
cp /mnt/ramdisk/masterkey /mnt/ramdisk/transferCD

#create iso from transferCD dir
genisoimage -r -J -o /mnt/ramdisk/transferCD.iso /mnt/ramdisk/transferCD

#wipe the CD
sudo umount $OUTPUT
wodim -v dev=$OUTPUT blank=fast

#burn transferCD iso to the transfer CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/transferCD.iso

#eject the disk to refresh the filesystem
eject $OUTPUT