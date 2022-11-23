#script for distributing 2 shards to SD card 3

sudo cp /mnt/ramdisk/setupCD/shards/shard3.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard3.txt
sudo cp /mnt/ramdisk/setupCD/shards/shard9.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard9.txt