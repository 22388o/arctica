FILE="/mnt/ramdisk/masterkey"
FILE1="/media/$USER/CDROM/masterkey"

if [ -f "$FILE" ]; then 
    #remove sensitive dir to be unpacked on next step
    sudo rm -r /mnt/ramdisk/sensitive
    echo "masterkey found"
else if [ -f "$FILE1" ]; then
    cp /media/$USER/CDROM/masterkey /mnt/ramdisk
    echo "masterkey found"
else
    echo "no key found"
fi