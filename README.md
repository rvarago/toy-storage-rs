# tokio-codec-experiment

My toy wire protocol implemented in async Rust.

> DISCLAIMER: This is nothing more than a **playground** project.

## Protocol

### SET

* Request: `SET <KEY> <VALUE>\n`
* Response: `OKAY <KEY>\n`
  
### GET

* Request: `GET <KEY>\n`
* Response (Success): `OKAY <KEY> <VALUE>\n`
* Response (Failure): `FAIL <KEY> <VALUE>\n`

## Example

Running an `nc` client against this project:

```
λ nc 127.0.0.1 8080
GET rafael
FAIL rafael
SET rafael 123
OKAY rafael
GET rafael
OKAY rafael 123
```