#this script will run after the tails flash on an SD
#it will set up the persistent partition on each SD without the user having to boot in and do it manually
echo "running setuppersistence"




# Copyright (c) 2022 Ben Westgate
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
# THE SOFTWARE.
#
# Parameter $1 = passphrase
# Parameter $2 = iteration time (ms) to unlock persistence
​
readonly TAILS_PART=$(mount | grep /lib/live/mount/medium | cut -f1 -d' ')
readonly DEVICE=${TAILS_PART%?}	# strips partition number off device
readonly SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
ITER_TIME=$2	# time to open the volume in milliseconds, longer iteration is more costly to crack.
readonly MEMORY=$(($(awk '/MemAvailable/{print $2}' /proc/meminfo) * 15 / 16))	# memory kB used for key-stretching, more is more costly to crack, but systems with < value used will not be able to open the encryption
​
clear -x
​
​
echo "#!/usr/bin/env bash
zenity --password --title='Enter Admin Password'" > $SCRIPT_DIR/askpass.sh
export SUDO_ASKPASS=$SCRIPT_DIR/askpass.sh
chmod +x $SCRIPT_DIR/askpass.sh
sudo --askpass mv /etc/sudoers.d/always-ask-password /etc/sudoers.d/always-ask-password.bak
echo "n
​
​
​
t
​
27
w" | sudo --askpass fdisk $DEVICE		# creates linux reserved partition in free-space
echo "name
2
TailsData
q" | sudo parted $DEVICE
​
# Sets up LUKS2 volume and file system on device then mounts it.
printf "$1" | sudo cryptsetup luksFormat --batch-mode --verbose --pbkdf=argon2id --iter-time=$ITER_TIME --pbkdf-memory $MEMORY --batch-mode "${DEVICE}2"
printf "$1" | sudo cryptsetup --verbose open "${DEVICE}2" TailsData_unlocked
sudo mkfs.ext4 -F -L 'TailsData' /dev/mapper/TailsData_unlocked
sudo mkdir --parents /live/persistence/TailsData_unlocked
sudo mount /dev/mapper/TailsData_unlocked /live/persistence/TailsData_unlocked
sudo rsync -PaSHAXv --del $SCRIPT_DIR/TailsData/ /live/persistence/TailsData_unlocked
sudo xdg-open /live/persistence/TailsData_unlocked
sudo umount /live/persistence/TailsData_unlocked
sudo cryptsetup close TailsData_unlocked
sudo mv /etc/sudoers.d/always-ask-password.bak /etc/sudoers.d/always-ask-password