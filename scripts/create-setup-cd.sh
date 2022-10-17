OUTPUT=$(echo $(ls /dev/sr?))

mkdir setupCD

sudo cp ~/config.txt ~/arctica/setupCD

sudo rm ~/config.txt

genisoimage -r -J -o setupCD.iso ~/arctica/setupCD

wodim dev=$OUTPUT -v -data setupCD.iso

sudo rm -r setupCD

sudo rm setupCD.iso

#sudo mount $OUTPUT /media/$USER/CDROM -o loop