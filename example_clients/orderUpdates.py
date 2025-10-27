#!/usr/bin/python3.12
import getopt
import stomper
import sys
import websocket


accountKey='noaccount'

def on_message(_, message):
    global accountKey
    # print(message)

    frame = stomper.Frame()
    unpacked_msg = stomper.Frame.unpack(frame, message)
    print("Received the application message: " + str(unpacked_msg))

def on_open(ws):
    ws.send("CONNECT\naccept-version:1.0,1.1,2.0\n\n"
            ""
            "\n")

    sub = stomper.subscribe("/accounts/" + accountKey + "/order_updates", 1, ack='auto')
    ws.send(sub)


def main(argv):
    global accountKey
    try:
        opts, args = getopt.getopt(argv, "", ["apiKey=","accountKey="])
    except getopt.GetoptError:
        print ('oops')
        sys.exit(2)
    for opt, arg in opts:
        if opt == '--apiKey':
            apiKey=arg
        if opt == '--accountKey':
            accountKey=arg

    websocket.enableTrace(True)

    ws_app = websocket.WebSocketApp("ws://192.168.111.107:8080/ws", cookie = 'api_key=' + apiKey, on_open=on_open, on_message=on_message)

    ws_app.run_forever(reconnect=1)


if __name__ == "__main__":
    main(sys.argv[1:])


# ws.connect("ws://localhost:5213/ws")
#
# client_id = 1
# sub = stom.javaper.subscribe("/topics/orders", client_id, ack='auto')
# ws.send(sub)
# ws.send("LTCBTC")
# while True:
#
#     result = ws.recv()
#     print ("Received '%s'" % result)
#
# ws.close()
