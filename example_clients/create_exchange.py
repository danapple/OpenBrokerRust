#!/usr/bin/python3.8
import getopt
import sys
import requests
from login import login
import time

def main(argv):
    api_key=''
    code=''
    description=''
    exchangeUrl=''
    websocketUrl=''
    exchangeApiKey=''
    try:
       opts, args = getopt.getopt(argv, "", ["apiKey=","code=", "description=", "exchangeUrl=", "websocketUrl=", "exchangeApiKey="])
    except getopt.GetoptError:
       print ('oops')
       sys.exit(2)
    for opt, arg in opts:
         if opt == '--apiKey':
             apiKey=arg
         if opt == '--code':
             code=arg
         if opt == '--exchangeUrl':
             exchangeUrl=arg
         if opt == '--websocketUrl':
             websocketUrl=arg
         if opt == '--description':
             description=arg
         if opt == '--exchangeApiKey':
             exchangeApiKey=arg

    (url, _, session) = login(apiKey)

    req = { "code": code, \
            "description": description, \
            "url": exchangeUrl, \
            "websocket_url": websocketUrl, \
            "api_key": exchangeApiKey
    }

    path=url + "/admin/exchange"

    print ('Requesting at path', path)
    print ('req', req)

    r = session.post(path, json=req, verify=False)

    print ('Create exchange Response')
    print(r)
    print(r.json())

if __name__ == "__main__":
    main(sys.argv[1:])
