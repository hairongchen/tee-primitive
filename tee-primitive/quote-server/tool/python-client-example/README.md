## compile proto file 

`python3 -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=.  quote_server.proto`

## run the client
`python3 client.py`
