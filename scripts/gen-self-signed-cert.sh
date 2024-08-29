# keyout - private key, out - public certificate.

#openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout key.pem -out cert.pem

openssl ecparam -genkey -name prime256v1 -out ec.key
openssl req -new -key ec.key -out ec.csr
openssl x509 -req -days 365 -in ec.csr -signkey ec.key -out ec.crt
