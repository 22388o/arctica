#timeout for click happy user
sleep 3

#remove old config from iso
sudo rm /media/$USER/writable/upper/home/ubuntu/config.txt

#copy over new config
sudo cp ~/config.txt /media/$USER/writable/upper/home/ubuntu/
sudo chmod 777 /media/$USER/writable/upper/home/ubuntu/config.txt

#remove current working config from local
sudo rm ~/config.txt

#burn iso
printf '%s\n' n y g y | mkusb ~/arctica/persistent-ubuntu.iso
