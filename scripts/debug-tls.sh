# Debug tls handshake.
# Example use: ./debug-tls.sh zenux.ru:443

openssl s_client -connect $1 -prexit
