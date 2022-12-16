sudo add-apt-repository -y universe

sudo apt update

#download wodim
sudo apt install -y wodim

#download shamir secret sharing library
#this needs to be made available on all 7 sd
sudo apt install ssss

#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#create setupCD config
echo "type=setupcd" > /mnt/ramdisk/CDROM/config.txt


#generate masterkey for encrypting persistent directories
base64 /dev/urandom | head -c 50 > /mnt/ramdisk/CDROM/masterkey

#split masterkey used for encryption into a 5 of 11 scheme
ssss-split -t 5 -n 11 < /mnt/ramdisk/CDROM/masterkey > /mnt/ramdisk/shards_untrimmed.txt
#make target dir for shard files
mkdir /mnt/ramdisk/shards


#trim excess from the output of ssss split
sed -e '1d' /mnt/ramdisk/shards_untrimmed.txt > /mnt/ramdisk/shards.txt
FILE="/mnt/ramdisk/shards.txt"
Lines=$(cat $FILE)
X=1
declare -i X
for Line in $Lines
do

    echo $Line > /mnt/ramdisk/shards/shard$X.txt
    X+=1
done

#NOTE: BEFORE THESE ARE REMOVED IN PROD THE APPROPRIATE SHARDS NEED TO GO TO THE BPS
#CONSIDER EVENTUALLY BREAKING THIS INTO A SEPERATE SCRIPT

#copy first 2 shards to SD 1
sudo cp /mnt/ramdisk/shards/shard1.txt /home/$USER/shards
sudo cp /mnt/ramdisk/shards/shard11.txt /home/$USER/shards


#remove stale shard file
sudo rm /mnt/ramdisk/shards_untrimmed.txt

#stage setup CD with shards for distribution to respective SD cards
sudo cp -R /mnt/ramdisk/shards /mnt/ramdisk/CDROM

#create iso from setupCD dir
genisoimage -r -J -o /mnt/ramdisk/setupCD.iso /mnt/ramdisk/CDROM

#wipe the CD
sudo umount $OUTPUT
wodim -v dev=$OUTPUT blank=fast

#burn setupCD iso to the Setup CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/setupCD.iso

#eject the disk to refresh the file system
eject $OUTPUT

