FILE="/mnt/ramdisk/masterkey"

if [ -f "$FILE" ]; then 
    echo "masterkey found"
else
    echo "no key found"
fi