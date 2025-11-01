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
    url=''
    websocketUrl=''
    exchangeApiKey=''
    try:
       opts, args = getopt.getopt(argv, "", ["apiKey=","code="])
    except getopt.GetoptError:
       print ('oops')
       sys.exit(2)
    for opt, arg in opts:
         if opt == '--apiKey':
             apiKey=arg
         if opt == '--code':
             code=arg

    (url, _, session) = login(apiKey)

    req = {  }

    path=url + "/admin/exchange/" + code

    print ('Requesting at path', path)
    print ('req', req)

    r = session.put(path, json=req, verify=False)

    print ('Create exchange Response')
    print(r)
    print(r.json())

if __name__ == "__main__":
    main(sys.argv[1:])
