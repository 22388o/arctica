#script for distributing 2 shards to an SD card

PLACEHOLDER=$(ls /mnt/ramdisk/setupCD/shards)
strarr=($PLACEHOLDER)
X=1
Y=3
declare -i X
for val in "${strarr[@]}";
do
    if [ $X -ne $Y ]; 
    then
    cp /mnt/ramdisk/setupCD/shards/$val /home/$USER
    sudo rm /mnt/ramdisk/setupCD/shards/$val
    X+=1
    else
    echo passing 
    fi
done