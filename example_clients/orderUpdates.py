#!/usr/bin/python3.12
import getopt
import stomper
import sys
import websocket
from login import login

accountKey='noaccount'

def on_message(_, message):
    global accountKey

    frame = stomper.Frame()
    unpacked_msg = stomper.Frame.unpack(frame, message)
    print("Received the application message: " + str(unpacked_msg))

def on_open(ws):
    ws.send("CONNECT\naccept-version:1.0,1.1,2.0\n\n"
            ""
            "\n")

    sub = stomper.subscribe("/accounts/" + accountKey + "/updates", 1, ack='auto')
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

    (url, addr, session) = login(apiKey)
    ws_app = websocket.WebSocketApp("ws://" + addr + "/ws", cookie = 'id=' + session.cookies.get('id'), on_open=on_open, on_message=on_message)

    ws_app.run_forever(reconnect=1)

if __name__ == "__main__":
    main(sys.argv[1:])
