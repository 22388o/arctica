#download wodim
sudo apt install -y wodim

#find cd path
OUTPUT=$(echo $(ls /dev/sr?))

#make the setup CD dir which holds files to be burned to the setup CD
mkdir setupCD

#copy setupCD config to the directory
sudo cp /home/$USER/config.txt /home/$USER/setupCD
sudo rm /home/$USER/config.txt

#generate SSH key for encrypting persistent directories and store in setupCD
ssh-keygen -t rsa -N '' -b 4096 -C "your_email@example.com" -f /home/$USER/setupCD/ssh_key

#create iso from setupCD dir
genisoimage -r -J -o setupCD.iso /home/$USER/setupCD

#burn setupCD iso to the Setup CD
wodim dev=$OUTPUT -v -data setupCD.iso
sudo rm -r setupCD
sudo rm setupCD.iso

#mount setup CD
sudo mount $OUTPUT /media/$USER/CDROM -o loop