#this script will execute when the user clicks the 'install' button on the boot screen
#this will generate our ssh key which will be used to encrypt persistent directories on each SD card

echo "generating ssh key for encryption"

#output needs to be changed to the setup CD here?
ssh-keygen -t rsa -N '' -b 4096 -C "your_email@example.com" -f $HOME/.ssh/id_rsa