#script for distributing 2 shards to SD card 4

sudo cp /mnt/ramdisk/shards/shard4.txt /home/$USER/shards
sudo rm /mnt/ramdisk/shards/shard4.txt
sudo cp /mnt/ramdisk/shards/shard8.txt /home/$USER/shards
sudo rm /mnt/ramdisk/shards/shard8.txt