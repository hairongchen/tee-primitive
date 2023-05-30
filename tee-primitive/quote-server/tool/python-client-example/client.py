import grpc
import quote_server_pb2 as pb2
import quote_server_pb2_grpc as pb2_grpc
import base64

def run():
    conn = grpc.insecure_channel('127.0.0.1:6789')

    report_data_string = "123456781234567812345678123456781234567812345678"
    report_data_string_bytes = report_data_string.encode("ascii")
    report_data_base64 = base64.b64encode(report_data_string_bytes)
    print(f"Encoded string: {report_data_base64}")

    client = pb2_grpc.GetQuoteStub(channel=conn)
    response = client.GetQuote(pb2.GetQuoteRequest(
        report_data = report_data_base64
    ))
    print(response.quote)


if __name__ == '__main__':
    run()
