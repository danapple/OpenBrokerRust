#!/usr/bin/python3.12

import getopt
import sys

import requests


def main(argv):
    customer_key=''

    try:
       opts, args = getopt.getopt(argv, "", ["accountKey="])
    except getopt.GetoptError:
       print ('oops')
       sys.exit(2)
    for opt, arg in opts:
         if opt == '--accountKey':
             accountKey=arg


    path="http://localhost:8080/accounts/" + accountKey + "/orders"
    r = requests.get(path, verify=False)

    print ('Response')
    print(r)
    print(r.json())
    print ('End of response')

if __name__ == "__main__":
    main(sys.argv[1:])
