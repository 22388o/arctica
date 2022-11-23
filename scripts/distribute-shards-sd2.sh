#script for distributing 2 shards to SD card 2

sudo cp /mnt/ramdisk/setupCD/shards/shard2.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard2.txt
sudo cp /mnt/ramdisk/setupCD/shards/shard10.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard10.txt