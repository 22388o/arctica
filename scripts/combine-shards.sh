rm /mnt/ramdisk/shards.txt
#combine a minimum of 5 numbered shard files in the shards dir into a single shard.txt file which can be accepted by ssss-combine
#/mnt/ramdisk/shards
PLACEHOLDER=$(ls /mnt/ramdisk/shards)
strarr=($PLACEHOLDER)
X=1
Y=6
declare -i X
for val in "${strarr[@]}";
do
    if [ $X -ne $Y ]; 
    then
    Line=$(cat /mnt/ramdisk/shards/$val)
    echo $Line >> /mnt/ramdisk/shards.txt
    X+=1
    else
    echo passing 
    fi
done

#once all 5 shards are in a single file (shards.txt) and properly formatted...
#combine 5 key shards inside of shards.txt to retrieve masterkey
ssss-combine -t 5 < /mnt/ramdisk/shards.txt 2> /mnt/ramdisk/masterkey_untrimmed.txt
FILE=$(cat /mnt/ramdisk/masterkey_untrimmed.txt)
#trim excess from reconstituted key
echo $FILE | cut -c 19- > /mnt/ramdisk/CDROM/masterkey
rm /mnt/ramdisk/masterkey_untrimmed.txt

