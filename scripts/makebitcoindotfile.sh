#this file will automatically create a .bitcoin directory in the users local home directory
#currently issues with this not creating the sub directories...
echo "running makebitcoindotfile"
sudo mkdir --parents /home/$USER/.bitcoin/blocks /home/$USER/.bitcoin/chainstate

# if [[ -d ../../../.bitcoin ]]
# then
#     echo "dotfile already exists"
# else
# then
#     echo "creating dotfile"
#     mkdir -parents ../../../.bitcoin/chainstate && mkdir blocks ../../../.bitcoin/
#     exit
# fi

# if [[ -d ../../../.bitcoin/chainstate ]]
# then
#     echo "chainstate already exists"
# else
# then
#     echo "creating chainstate"
#     mkdir chainstate ../../../.bitcoin/
# fi

# if [[ -d ../../../.bitcoin/blocks ]]
# then
#     echo "blocks already exists"
# else
# then
#     echo "creating blocks"
#     mkdir blocks ../../../.bitcoin 
# fi