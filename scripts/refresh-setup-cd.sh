#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#create iso from setupCD dir
genisoimage -r -J -o /mnt/ramdisk/setupCD.iso /mnt/ramdisk/setupCD

#wipe the CD
sudo umount $OUTPUT
wodim -v dev=$OUTPUT blank=fast

#burn setupCD iso to the Setup CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/setupCD.iso

#eject the disk to refresh the filesystem
eject $OUTPUT