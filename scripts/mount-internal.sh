#mount internal drive
sudo mount nvme0n1p2 /media/ubuntu
#remove stale symlinks
sudo unlink ~/.bitcoin/chainstate
sudo unlink ~/.bitcoin/blocks
#create symlinks for chainstate and blockdata
sudo ln -s /media/ubuntu/home/$USER/.bitcoin/chainstate ~/.bitcoin/chainstate
sudo ln -s /media/ubuntu/home/$USER/.bitcoin/blocks ~/.bitcoin/blocks
