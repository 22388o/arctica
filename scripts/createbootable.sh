#this script will execute when the user clicks the 'continue' button on each of the first setup screens for all 7 SD cards

target=$(ls /dev/sd?)

len=$(echo -n "$target" | wc -c)

if [[ $len -lt 8 ]]
then
echo no target device found, try again
elif [[ $len -lt 9 ]]
then
sudo dd if=tails-amd64-5.4.img of="$target" bs=16M oflag=direct status=progress
else
echo more than one target device found, confused about what to do
fi