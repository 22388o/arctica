#create a simulated xpriv
echo "this is a simulated xpriv" > /mnt/ramdisk/sensitive/xpriv.txt
#create a simulated xpub
echo "this is a simulated xpub" > /mnt/ramdisk/sensitive/xpub.txt

mkdir --parents /mnt/ramdisk/setupCD/xpubs

#export xpub to setupCD
cp /mnt/ramdisk/sensitive/xpub.txt /mnt/ramdisk/setupCD/xpubs/