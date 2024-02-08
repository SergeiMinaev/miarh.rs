# Private key (-inkey arg) and public certificate (-in arg) are required. 
# You can generate self-signed certs with scripts/gen-self-signed-cert.sh.
# Example use: ./wrap-pem-to-pfx.sh privkey.pem pubcert.pem

openssl pkcs12 -inkey $1 -in $2 -export -out identity.pfx
