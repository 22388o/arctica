#ENCRYPT DATA ON SD CARDS

#format shards correctly into a single file
#01-key
#02-key
#03-key
#04-key
#05-key

#combine masterkey shards to obtain masterkey
#combine-masterkey.sh

#pack sensitive data directory into a tarball
#packup.sh
tar cvf unencrypted_output.tar unencrypted

#encrypt tarball with gpg, using masterkey
gpg --batch --passphrase-file ~/testkey --output encrypted_output_tarball.gpg --symmetric unencrypted_output.tar


#DECRYPT DATA ON SD CARDS:   

#format shards correctly into a single file
#01-key
#02-key
#03-key
#04-key
#05-key

#combine masterkey shards to obtain masterkey
#combine-masterkey.sh

#decrypt tarball with gpg using masterkey
#unpack.sh
gpg --batch --passphrase-file testkey --output decrypted_tarball.out -d encrypted_output_tarball.gpg

#unpack sensitive data directory from tarball
tar xvf decrypted_tarball.out -C output



#NOTES:
#use this to append files to a decrypted tarball without having to create an entire new one
tar rvf output_tarball ~/filestobeappended