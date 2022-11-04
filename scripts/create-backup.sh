#make backup dir for iso
mkdir /mnt/ramdisk/backup

#copy shards
cp -r /home/$USER/shards /mnt/ramdisk/backup
#copy sensitive dir
cp /home/$USER/encrypted.gpg /mnt/ramdisk/backup
#create config
#eventually make this append the SD number to the config definition
echo "TYPE=Backup1" >> /mnt/ramdisk/backup/config.txt
#copy btc core
cp -r /home/$USER/bitcoin-22.0 /mnt/ramdisk/backup
#copy binary
cp /home/$USER/arctica /mnt/ramdisk/backup
#copy image
cp /home/$USER/arctica.jpeg /mnt/ramdisk/backup
#copy scripts
cp -r /home/$USER/scripts /mnt/ramdisk/backup