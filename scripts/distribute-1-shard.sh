#script for distributing 1 shard to an SD card

PLACEHOLDER=$(ls /mnt/ramdisk/shards)
strarr=($PLACEHOLDER)
X=1
Y=2
declare -i X
for val in "${strarr[@]}";
do
    if [ $X -ne $Y ]; 
    then
    cp /mnt/ramdisk/shards/$val /home/$USER
    rm /mnt/ramdisk/shards/$val
    X+=1
    else
    echo passing 
    fi
done