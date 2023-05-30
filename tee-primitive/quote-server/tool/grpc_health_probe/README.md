## download and build grpc-health-probe

`git clone https://github.com/grpc-ecosystem/grpc-health-probe.git`
`go build`

## check server health
`./grpc-health-probe -addr localhost:6789`
```
status: SERVING
```

