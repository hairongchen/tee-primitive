## download grpcurl binary

`wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.7/grpcurl_1.8.7_linux_x86_64.tar.gz`

## run the grpcurl to list supported remote grpc calls
`./grpcurl -plaintext localhost:6789 list`

```
grpc.reflection.v1alpha.ServerReflection
quoteserver.GetQuote
quoteserver.Health
```
## describe a grpc call
`./grpcurl -plaintext localhost:6789 describe quoteserver.GetQuote`
```
quoteserver.GetQuote is a service:
service GetQuote {
  rpc GetQuote ( .quoteserver.GetQuoteRequest ) returns ( .quoteserver.GetQuoteResponse );
}
```
## access a grpc call
`./grpcurl -d '{"report_data": "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4"}' -plaintext localhost:6789  quoteserver.GetQuote.GetQuote`
