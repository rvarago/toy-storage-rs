# A **toy** in-memory storage

This is a toy in-memory storage server with data exchanged over the network.

> DISCLAIMER: This is nothing more than a self-contained **playground** project for having some fun while learning Rust.

## Wire Protocol

This is a request-response protocol over TCP, where the client initiates an exchange by sending a request to the server which then sends a response back to the client.

Messages (request/response) are line-delimited.

### SET

* Request: `SET <KEY> <VALUE>\n`
* Response: `OKAY <KEY>\n`
  
### GET

* Request: `GET <KEY>\n`
* Response (Success): `OKAY <KEY> <VALUE>\n`
* Response (Failure): `FAIL <KEY> <VALUE>\n`

## Example Session

By simulating a client as an `nc` instance:

```bash
λ nc 127.0.0.1 8080
GET rafael
FAIL rafael
SET rafael 123
OKAY rafael
GET rafael
OKAY rafael 123
```
