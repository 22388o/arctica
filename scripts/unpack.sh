#decrypt sensitive directory
gpg --batch --passphrase-file /mnt/ramdisk/masterkey --output /mnt/ramdisk/decrypted.out -d /home/$USER/encrypted.gpg

#unpack sensitive directory tarball
tar xvf /mnt/ramdisk/decrypted.out -C /mnt/ramdisk/

#copy sensitive dir to ramdisk
cp -R /mnt/ramdisk/mnt/ramdisk/sensitive /mnt/ramdisk

#remove nested sensitive
sudo rm -r /mnt/ramdisk/mnt 

#NOTES:
#use this to append files to a decrypted tarball without having to create an entire new one
#tar rvf output_tarball ~/filestobeappended