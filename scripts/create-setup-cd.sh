#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

mkdir setupCD
sudo cp ~/config.txt ~/arctica/setupCD
sudo rm ~/config.txt

#create iso
genisoimage -r -J -o setupCD.iso ~/arctica/setupCD

#burn disc
wodim dev=$OUTPUT -v -data setupCD.iso
sudo rm -r setupCD
sudo rm setupCD.iso

#to mount
sudo mount $OUTPUT /media/$USER/CDROM -o loop