#generate masterkey for encrypting persistent directories
base64 /dev/urandom | head -c 50 > /mnt/ramdisk/CDROM/masterkey

#split masterkey used for encryption into a 5 of 11 scheme
ssss-split -t 5 -n 11 < /mnt/ramdisk/CDROM/masterkey > /mnt/ramdisk/shards_untrimmed.txt
#make target dir for shard files
mkdir /mnt/ramdisk/shards


#trim excess from the output of ssss split
sed -e '1d' /mnt/ramdisk/shards_untrimmed.txt > /mnt/ramdisk/shards.txt
FILE="/mnt/ramdisk/shards.txt"
Lines=$(cat $FILE)
X=1
declare -i X
for Line in $Lines
do

    echo $Line > /mnt/ramdisk/shards/shard$X.txt
    X+=1
done

