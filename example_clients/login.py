import requests


addr = "192.168.111.107:8080"
url = "http://" + addr

def login(apiKey):
    session = requests.Session()

    login_req = { "api_key" : apiKey }
    login_path=url + "/loginapi"
    print ('Requesting login at path', login_path)
#     print ('login req', login_req)
    login_resp = session.post(login_path, json=login_req, verify=False)

    idCookie = session.cookies.get('id')
    session.cookies.clear()
# Get rid of the secure flag; useful when using http
    session.cookies.set('id', value=idCookie)

    return (url, addr, session)
