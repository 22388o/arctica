#decrypt sensitive directory
gpg --batch --passphrase-file /mnt/ramdisk/masterkey --output /mnt/ramdisk/decrypted.out -d /home/$USER/encrypted.gpg

#unpack sensitive directory tarball
tar xvf /mnt/ramdisk/decrypted.out -C /mnt/ramdisk/