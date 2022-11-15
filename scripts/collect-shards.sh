#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#make the transfer CD dir which holds files to be burned to the transfer CD
#maybe break this out into a seperate rust function so that it can exist for the config write
mkdir /mnt/ramdisk/transferCD

#stage transfer CD with shards from current SD and from existing transfer CD ISO if applicable
cp -R /media/$USER/CDROM/* /mnt/ramdisk/transferCD

#create transferCD config
#perhaps make this step only available in a 'make recovery cd' script, will cause issues here if we reuse this script more than once
#will probably need this to be a seperate rust function
# echo "type=transfercd" > /mnt/ramdisk/transferCD/config.txt
# echo "recoveryStep=X" > /mnt/ramdisk/transferCD/config.txt

#collect shards from sd card for export to transfer CD
cp -r /home/$USER/shards/* /mnt/ramdisk/transferCD/shards

#create iso from transferCD dir
genisoimage -r -J -o /mnt/ramdisk/transferCD.iso /mnt/ramdisk/transferCD

#wipe the CD
sudo umount $OUTPUT
wodim -v dev=$OUTPUT blank=fast

#burn transferCD iso to the transfer CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/transferCD.iso

#eject the disk to refresh the filesystem
eject $OUTPUT