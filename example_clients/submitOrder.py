#!/usr/bin/python3.8
import getopt
import sys
import requests
from login import login

def main(argv):
    api_key=''
    price=''
    quantity=''
    instrumentId=''
    client_order_id=''
    try:
       opts, args = getopt.getopt(argv, "", ["instrumentId=","price=","quantity=","accountKey=","apiKey="])
    except getopt.GetoptError:
       print ('oops')
       sys.exit(2)
    for opt, arg in opts:
         if opt == '--price':
             price = arg
         if opt == '--quantity':
             quantity=arg
         if opt == '--instrumentId':
             instrumentId=arg
         if opt == '--accountKey':
             accountKey=arg
         if opt == '--apiKey':
             apiKey=arg

    (url, _, session) = login(apiKey)

    req = { "price": float(price), \
                           "quantity": int(quantity),
                           "legs": [ \
                               {"ratio": 1, "instrument_id": int(instrumentId)} \
                               ]\
            }

    path=url + "/accounts/" + accountKey + "/orders"

    print ('Requesting at path', path)
    print ('req', req)

    r = session.post(path, json=req, verify=False)

    print ('Submit order Response')
    print(r)
    print(r.json())

if __name__ == "__main__":
    main(sys.argv[1:])
