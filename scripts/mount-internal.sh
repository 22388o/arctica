#the below internal drive configurations assume a default ubuntu install on the internal disk without any 
#custom partitioning
#mount internal drive if nvme
udisksctl mount --block-device /dev/nvme0n1p2
#mount internal drive if SATA
udisksctl mount --block-device /dev/sda2
#remove stale symlinks
#this unlink corrupts the chainstate if core is currently running
#/home/user/bitcoin23/bin/bitcoin-cli stop
#sleep 20 (or query bitcoin-cli until shutdown is done)
#when querying bitcoin-cli while bitcoind is not running receive the following error: 
#error: timeout on transient error: Could not connect to the server 127.0.0.1:8332
#Make sure the bitcoind server is running and that you are connvecting to the correct RPC port. 
sudo chmod -r 777 /home/$USER/.bitcoin
sudo unlink /home/$USER/.bitcoin/chainstate
sudo unlink /home/$USER/.bitcoin/blocks

# UUID=$(echo $(blkid) | cut -d '"' -f 2)

#obtain Host User and UUID mounted by udisksctl
PLACEHOLDER=$(ls /media/$USER)
strarr=($PLACEHOLDER)


#loop through every item in /media/$USER and check the value length. If it's 36 characters can assume UUID and assign value.
for val in "${strarr[@]}";
do
    if	[[ ${#val} -eq 36 ]]
    then
            echo $val
            UUID=$val
    else
            echo pass $val
    fi

done

#define the username of the internal storage device
HOST_USER=$(ls /media/$USER/$UUID/home)
#open file permissions for local host
sudo chmod 777 /media/ubuntu/$UUID/home/$HOST_USER

#remove stale bitcoin dirs if they exist
sudo rm -r /home/$USER/.bitcoin/chainstate
sudo rm -r /home/$USER/.bitcoin/blocks

#make local internal bitcoin dotfile
sudo mkdir --parents /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin/blocks /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin/chainstate	

#create symlinks
ln -s /media/$USER/$UUID/home/$HOST_USER/.bitcoin/chainstate /home/$USER/.bitcoin
ln -s /media/$USER/$UUID/home/$HOST_USER/.bitcoin/blocks /home/$USER/.bitcoin

sudo chmod -R 777 /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin

