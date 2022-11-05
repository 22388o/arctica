#make backup dir for iso
mkdir /mnt/ramdisk/backup

#copy shards
cp -r /home/$USER/shards /mnt/ramdisk/backup
#copy sensitive dir
cp /home/$USER/encrypted.gpg /mnt/ramdisk/backup
#create config
#eventually make this append the SD number to the config definition
echo "TYPE=Backup1" >> /mnt/ramdisk/backup/config.txt

#create iso from backup CD dir
#eventually make this append SD number to the iso name
genisoimage -r -J -o /mnt/ramdisk/backupSDNumber.iso /mnt/ramdisk/backup