# Rust Home Library

Allows you to catalog the books you keep at home via a scanner accessible on your phone. Uses a SQLite database to store records.

# Setup

In order to function, the Rust Home Library needs a (self-)signed SSL certificate to enable camera use in a mobile browser. It also utilizes a SQLite database on the backend, but this shouldn't require installation, as the relevant Rust crate should handle this. However, if you want to manually access the database, you will need to install SQLite.

## SSL Certificate Setup

Before starting, ensure you have OpenSSL installed. Then run

`openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 365 -nodes \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1,IP:192.168.1.50"`

Replace `192.168.1.50` with the LAN IP address of the machine running the Rust Home Library. Place the generated `key.pem` and `cert.pem` files into the root directory of the project. Now with a self-signed certificate, you should be able to connect to the scanner site via https, and as a result, be able to use your camera. Since this is a self-signed certifcate, your browser will likely warn you of this, and check to make sure you want to proceed. Your browser should have a way of getting around this warning. In Chrome for example, you press the "Advanced settings" option at the bottom of the warning page, and then hit the "proceed" link that is now visible.

When accessing the site, make sure you preface the address with https. For example, `https://192.168.1.50:3000`.
