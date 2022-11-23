#script for distributing 2 shards to SD card 4

sudo cp /mnt/ramdisk/setupCD/shards/shard4.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard4.txt
sudo cp /mnt/ramdisk/setupCD/shards/shard8.txt /home/$USER/shards
sudo rm /mnt/ramdisk/setupCD/shards/shard8.txt