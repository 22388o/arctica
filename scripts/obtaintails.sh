#this script will obtain the latest tails image 
#eventually we will verify the signatures here as well

echo "grabbing the latest build of tails"

wget --continue http://dl.amnesia.boum.org/tails/stable/tails-amd64-5.4/tails-amd64-5.4.img 

echo $