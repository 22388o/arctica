#mount internal drive
udisksctl mount --block-device /dev/nvme0n1p2
#remove stale symlinks
sudo chmod 777 /home/$USER/.bitcoin
sudo unlink /home/$USER/.bitcoin/chainstate
sudo unlink /home/$USER/.bitcoin/blocks

#obtain Host User and UUID mounted by udisksctl
PLACEHOLDER=$(ls /media/$USER)
strarr=($PLACEHOLDER)

for val in "${strarr[@]}";
do
    if	[ "$val" = "writable" ]
    then
            echo pass
    elif  [ "$val" = "CDROM" ]
    then
            echo pass
    else
            echo $val
            UUID=$val
    fi

done
HOST_USER=$(ls /media/$USER/$UUID/home)
#open file permissions for local host
sudo chmod 777 /media/ubuntu/$UUID/home/$HOST_USER

#create symlinks
ln -s /media/$USER/$UUID/home/$HOST_USER/.bitcoin/chainstate /home/$USER/.bitcoin
ln -s /media/$USER/$UUID/home/$HOST_USER/.bitcoin/blocks /home/$USER/.bitcoin

sudo chmod -R 777 /media/ubuntu/$UUID/home/$HOST_USER/.bitcoin
#start bitcoind
# /home/$USER/bitcoin-23.0/bin/bitcoind
#this is bad for your RAM during initial setup and debugging mkay
