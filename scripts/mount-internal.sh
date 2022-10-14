#mount internal drive
# sudo mount /dev/nvme0n1p2 /media/ubuntu 
# #remove stale symlinks
# sudo unlink ~/.bitcoin/chainstate
# sudo unlink ~/.bitcoin/blocks
# #create symlinks for chainstate and blockdata
# ln -s /media/ubuntu/home/$USER/.bitcoin/chainstate ~/.bitcoin/chainstate
# ln -s /media/ubuntu/home/$USER/.bitcoin/blocks ~/.bitcoin/blocks
