#this file will automatically create a .bitcoin directory in the users local home directory
#currently issues with this not creating the sub directories...
echo "running makebitcoindotfile"
sudo mkdir --parents /home/$USER/.bitcoin/blocks /home/$USER/.bitcoin/chainstate
