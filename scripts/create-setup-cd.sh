sudo add-apt-repository -y universe

sudo apt update

#download wodim
sudo apt install -y wodim
#download shamir secret sharing
sudo apt install ssss

#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#make the setup CD dir which holds files to be burned to the setup CD
sudo mkdir /mnt/ramdisk/setupCD
sudo chmod 777 /mnt/ramdisk/setupCD

#create setupCD config
echo "type=setupcd" > /mnt/ramdisk/setupCD/config.txt


#generate masterkey for encrypting persistent directories and store in setupCD
base64 /dev/urandom | head -c 50 > /mnt/ramdisk/masterkey

#split masterkey used for encryption into a 5 of 11 scheme
ssss-split -t 5 -n 11 < /mnt/ramdisk/masterkey > /mnt/ramdisk/shards_untrimmed.txt
#make target dir for shard files
sudo mkdir /mnt/ramdisk/shards
sudo chmod 777 /mnt/ramdisk/shards

#trim excess from the output of ssss split
sed -e '1d' /mnt/ramdisk/shards_untrimmed.txt > /mnt/ramdisk/shards.txt
FILE="/mnt/ramdisk/shards.txt"
Lines=$(cat $FILE)
X=1
declare -i X
for Line in $Lines
do

    echo $Line | cut -c 4- > /mnt/ramdisk/shards/shard$X.txt
    X+=1
done

#NOTE: BEFORE THESE ARE REMOVED IN PROD THE APPROPRIATE SHARDS NEED TO GO TO THE BPS
#CONSIDER EVENTUALLY BREAKING THIS INTO A SEPERATE SCRIPT

#copy first 2 shards to SD 1
sudo cp /mnt/ramdisk/shards/shard1.txt /home/$USER
sudo rm /mnt/ramdisk/shards/shard1.txt
sudo cp /mnt/ramdisk/shards/shard2.txt /home/$USER
sudo rm /mnt/ramdisk/shards/shard2.txt


#remove stale shard file
sudo rm /mnt/ramdisk/shards_untrimmed.txt

#stage setup CD with masterkey (for decrypting without bps) and shards for distribution to SD cards
sudo cp /mnt/ramdisk/masterkey /mnt/ramdisk/setupCD
sudo cp /mnt/ramdisk/shards /mnt/ramdisk/setupCD

#create iso from setupCD dir
genisoimage -r -J -o /mnt/ramdisk/setupCD.iso /mnt/ramdisk/setupCD

#burn setupCD iso to the Setup CD
wodim dev=$OUTPUT -v -data /mnt/ramdisk/setupCD.iso

