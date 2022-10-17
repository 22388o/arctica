#mount internal drive
sudo mount /dev/nvme0n1p2 /media/ubuntu/InternalDisk
# #remove stale symlinks
sudo unlink ~/.bitcoin/chainstate
sudo unlink ~/.bitcoin/blocks
# #create symlinks for chainstate and blockdata
ln -s /media/ubuntu/InternalDisk/home/$USER/.bitcoin/chainstate ~/.bitcoin/chainstate
ln -s /media/ubuntu/InternalDisk/home/$USER/.bitcoin/blocks ~/.bitcoin/blocks
