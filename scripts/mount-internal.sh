#mount internal drive
sudo mount /dev/nvme0n1p2 /media/ubuntu
# #remove stale symlinks
# sudo unlink ~/.bitcoin/chainstate
# sudo unlink ~/.bitcoin/blocks
# #create symlinks for chainstate and blockdata

#these symlinks are currently broken because $USER is dynamic.
#it is not the $USER var but instead the user on host machine
#need to figure out how to parse this in for absolute path.

# ln -s /media/$USER/home/$HOST_USER/.bitcoin/chainstate ~/.bitcoin/chainstate
# ln -s /media/$USER/home/$HOST_USER/.bitcoin/blocks ~/.bitcoin/blocks