sudo add-apt-repository -y universe

sudo apt update

#download wodim
sudo apt install -y wodim
#download shamir secret sharing
sudo apt install ssss

#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#make the setup CD dir which holds files to be burned to the setup CD
mkdir /mnt/ramdisk/setupCD

#copy setupCD config to the directory
sudo cp /home/$USER/config.txt /mnt/ramdisk/setupCD
sudo rm /home/$USER/config.txt

#generate SSH key for encrypting persistent directories and store in setupCD
ssh-keygen -t rsa -N '' -b 4096 -C "your_email@example.com" -f /mnt/ramdisk/setupCD/masterkey
#split masterkey used for encryption into a 5 of 11 scheme
ssss-split -t 5 -n 11 < /mnt/ramdisk/setupCD/masterkey > /mnt/ramdisk/setupCD/shards.txt
sudo cp /mnt/ramdisk/masterkey /mnt/ramdisk/setupCD
sudo cp /mnt/ramdisk/shards.txt /mnt/ramdisk/setupCD

#create iso from setupCD dir
genisoimage -r -J -o /mnt/ramdisk/setupCD.iso /mnt/ramdisk/setupCD

#burn setupCD iso to the Setup CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/setupCD.iso

