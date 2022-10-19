#mount internal drive
sudo mount /dev/nvme0n1p2 /media/ubuntu
# #remove stale symlinks
sudo unlink /home/$USER/.bitcoin/chainstate
sudo unlink /home/$USER/.bitcoin/blocks
# #create symlinks for chainstate and blockdata
HOST_USER=$(ls /media/$USER/home)
sudo ln -s /media/$USER/home/$HOST_USER/.bitcoin/chainstate /home/$USER/.bitcoin/chainstate
sudo ln -s /media/$USER/home/$HOST_USER/.bitcoin/blocks /home/$USER/.bitcoin/blocks
