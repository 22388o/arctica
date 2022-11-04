#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#create iso from backup CD dir
#eventually make this append SD number to the iso name
genisoimage -r -J -o /mnt/ramdisk/backupSDNumber.iso /mnt/ramdisk/backup

#wipe the CD
sudo umount $OUTPUT
wodim -v dev=$OUTPUT blank=fast

#burn setupCD iso to the backup CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/backupSDNumber.iso

#eject the disk to refresh the filesystem
eject $OUTPUT