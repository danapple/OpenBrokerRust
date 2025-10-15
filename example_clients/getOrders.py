#!/usr/bin/python3.12

import getopt
import sys

import requests


def main(argv):
    path="http://localhost:8080/accounts/aaaa/orders"
    r = requests.get(path, verify=False)

    print ('Response')
    print(r)
    print(r.json())
    print ('End of response')

if __name__ == "__main__":
    main(sys.argv[1:])
