#combine a minimum of 5 numbered shard files in the shards dir into a single shard.txt file which can be accepted by ssss-combine
#/mnt/ramdisk/shards
PLACEHOLDER=$(ls /mnt/ramdisk/shards)
strarr=($PLACEHOLDER)
X=1
declare -i X

for val in "${strarr[@]}";
do
    Line=$(cat /mnt/ramdisk/shards/$val)
    echo 0$X-$Line >> /mnt/ramdisk/shards.txt
    X+=1

done

#ssss input file needs to be formatted as follows
#01-key
#02-key
#03-key
#04-key
#05-key


#once all 5 shards are in a single file (shards.txt) and properly formatted...
#combine 5 key shards inside of shards.txt to retrieve masterkey
ssss-combine -t 5 < /mnt/ramdisk/shards.txt 2> /mnt/ramdisk/masterkey_untrimmed.txt
FILE=$(cat /mnt/ramdisk/masterkey_untrimmed.txt)
#trim excess from reconstituted key
echo $FILE | cut -c 19- > /mnt/ramdisk/masterkey
rm /mnt/ramdisk/masterkey_untrimmed.txt

