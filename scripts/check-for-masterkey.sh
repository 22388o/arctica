FILE="/mnt/ramdisk/masterkey"

if [ -f "$FILE" ]; then 
    #remove sensitive dir to be unpacked on next step
    sudo rm -r /mnt/ramdisk/sensitive
    echo "masterkey found"
else
    echo "no key found"
fi