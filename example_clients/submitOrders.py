#!/usr/bin/python3.8
import getopt
import sys

import requests


def main(argv):

    api_key=''
    price=''
    quantity=''
    instrumentId=''
    client_order_id=''
    try:
       opts, args = getopt.getopt(argv, "", ["instrumentId=","price=","quantity=","accountKey="])
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


    req = { "price": float(price), \
                           "quantity": int(quantity),
                           "legs": [ \
                               {"ratio": 1, "instrument_id": instrumentId} \
                               ]\
            }
                           
                         

    cookies = { "api_key": api_key }

    #path="http://openexchange.eu-central-1.elasticbeanstalk.com/order/" + client_order_id
    path="http://localhost:8080/accounts/" + accountKey + "/orders"

    print ('Requesting at path', path)
    print ('req', req)

    r = requests.post(path , json=req, cookies=cookies, verify=False)

    print ('Response')
    print(r)
    print(r.json())


if __name__ == "__main__":
   main(sys.argv[1:])
