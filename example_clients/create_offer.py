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
    expiration_days=''
    try:
       opts, args = getopt.getopt(argv, "", ["apiKey=","code=", "description=", "expiration_days="])
    except getopt.GetoptError:
       print ('oops')
       sys.exit(2)
    for opt, arg in opts:
         if opt == '--apiKey':
             apiKey=arg
         if opt == '--code':
             code=arg
         if opt == '--description':
             description=arg
         if opt == '--expiration_days':
             expiration_days=arg


    expiration_time = int((time.time() * 1000) + (86400 * 1000 * int(expiration_days)))


    (url, _, session) = login(apiKey)

    req = { "code": code, \
            "description": description, \
            "expiration_time": expiration_time
    }

    path=url + "/admin/offer"

    print ('Requesting at path', path)
    print ('req', req)

    r = session.post(path, json=req, verify=False)

    print ('Submit order Response')
    print(r)
    print(r.json())

if __name__ == "__main__":
    main(sys.argv[1:])
