#combine atleast 5 key shards inside of shards.txt to retrieve masterkey
ssss-combine -t 5 < /mnt/ramdisk/shards.txt 2> /mnt/ramdisk/masterkey.txt

#shards must be formatted as follows
#01-key
#02-key
#03-key
#04-key
#05-key
