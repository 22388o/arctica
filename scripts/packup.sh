#remove stale encrypted dir
sudo rm /home/$USER/encrypted.gpg

#pack the sensitive directory into a tarball
tar cvf /mnt/ramdisk/unencrypted.tar /mnt/ramdisk/sensitive

#encrypt the sensitive directory tarball 
gpg --batch --passphrase-file /mnt/ramdisk/masterkey --output /home/$USER/encrypted.gpg --symmetric /mnt/ramdisk/unencrypted.tar

