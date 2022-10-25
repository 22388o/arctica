#pack target directory into a tarball
tar cvf unencrypted_output.tar unencrypted

#encrypt tarball with gpg, using masterkey
gpg --output encrypted_output_tarball.gpg --symmetric unencrypted_output.tar --passphrase-file testkey


#shard masterkey



#combine masterkey shards


#decrypt tarball
gpg --output decrypted_tarball.out -d encrypted_output_tarball.gpg

#unpack target directory
tar xvf decrypted_tarball.out



#NOTEs:
#use this to append files to a decrypted tarball
tar rvf output_tarball ~/filestobeappended